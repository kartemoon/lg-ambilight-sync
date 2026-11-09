use crate::{config::NanoleafConfig, local_http, runtime::RetryState};
use anyhow::{anyhow, Result};
use serde_json::json;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Off,
    Colour(String),
    Scene(String),
}

pub fn desired(
    enabled: bool, was_managed: bool, active: Option<bool>, display_off: bool,
    scene: &str, colour: &str,
) -> Option<Action> {
    if !enabled {
        return was_managed.then_some(Action::Off);
    }
    match (active, display_off) {
        (Some(false), _) | (_, true) => Some(Action::Off),
        (Some(true), false) if scene.is_empty() => Some(Action::Colour(colour.into())),
        (Some(true), false) => Some(Action::Scene(scene.into())),
        (None, false) => None,
    }
}

pub struct IdleLighting {
    applied: Option<Action>,
    retry: RetryState,
    standby_epoch: u64,
}

impl Default for IdleLighting {
    fn default() -> Self { Self { applied: None, retry: RetryState::new(), standby_epoch: 0 } }
}

impl IdleLighting {
    pub fn observe_standby(&mut self, epoch: u64) -> bool {
        if self.standby_epoch == epoch { return false; }
        self.standby_epoch = epoch;
        self.applied = None;
        self.retry.success();
        true
    }

    pub fn was_managed(&self) -> bool {
        matches!(self.applied, Some(Action::Colour(_) | Action::Scene(_)))
    }

    pub fn capture_resumed(&mut self) -> bool {
        let controlled = self.applied.take().is_some();
        self.retry.success();
        controlled
    }

    pub async fn update(&mut self, device: Option<&NanoleafConfig>, action: Option<Action>) {
        let (Some(device), Some(action)) = (device, action) else { return };
        if self.applied.as_ref() == Some(&action) || !self.retry.ready() { return; }
        let device = device.clone();
        let request = action.clone();
        match tokio::task::spawn_blocking(move || apply(&device, &request)).await {
            Ok(Ok(())) => {
                tracing::info!("Режим Nanoleaf без захвата: {:?}", action);
                self.applied = Some(action);
                self.retry.success();
            }
            Ok(Err(error)) => {
                let delay = self.retry.failure();
                tracing::warn!("Фоновая подсветка Nanoleaf: {error}. Повтор через {delay:?}");
            }
            Err(error) => {
                self.retry.failure();
                tracing::warn!("Не удалось запустить управление фоновой подсветкой: {error}");
            }
        }
    }
}

fn endpoint(device: &NanoleafConfig, path: &str) -> String {
    format!("http://{}:16021/api/v1/{}{}", device.ip, device.auth_token, path)
}

fn put(device: &NanoleafConfig, path: &str, body: serde_json::Value) -> Result<()> {
    local_http::agent(Some(Duration::from_millis(1200)))
        .put(endpoint(device, path))
        .send_json(body)
        .map_err(|error| local_http::request_error("Nanoleaf background lighting", error))?;
    Ok(())
}

pub fn power(device: &NanoleafConfig, on: bool) -> Result<()> {
    put(device, "/state", json!({"on":{"value":on}}))
}

pub fn prepare_capture(device: &NanoleafConfig) -> Result<()> {
    put(device, "/state", json!({"on":{"value":true},"brightness":{"value":100}}))
}

pub fn scenes(device: &NanoleafConfig) -> Result<Vec<String>> {
    let mut names: Vec<String> = local_http::agent(Some(Duration::from_secs(3)))
        .get(endpoint(device, "/effects/effectsList"))
        .call()
        .map_err(|error| local_http::request_error("Nanoleaf scenes", error))?
        .into_body()
        .with_config().limit(64 * 1024).read_json()
        .map_err(|_| anyhow!("Не удалось прочитать список сцен Nanoleaf"))?;
    names.retain(|name| !name.trim().is_empty() && name.len() <= 128 && !name.chars().any(char::is_control));
    names.sort_unstable();
    names.dedup();
    Ok(names)
}

pub fn valid_colour(colour: &str) -> bool {
    colour.len() == 7 && colour.starts_with('#') && colour.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
}

fn colour_state(colour: &str) -> Result<serde_json::Value> {
    if !valid_colour(colour) { return Err(anyhow!("Некорректный цвет подсветки")); }
    let hex = u32::from_str_radix(&colour[1..], 16)?;
    let [r, g, b] = [16, 8, 0].map(|shift| ((hex >> shift) & 255u32) as f64 / 255.0);
    let max = r.max(g).max(b);
    let delta = max - r.min(g).min(b);
    if max == 0.0 { return Ok(json!({"on":{"value":false}})); }
    let h = if delta == 0.0 { 0.0 } else if max == r { (g-b)/delta }
        else if max == g { (b-r)/delta + 2.0 } else { (r-g)/delta + 4.0 };
    Ok(json!({"on":{"value":true}, "hue":{"value":(h*60.0).rem_euclid(360.0).round() as u16 % 360},
        "sat":{"value":(delta/max*100.0).round() as u8}, "brightness":{"value":(max*100.0).round().clamp(1.0, 100.0) as u8}}))
}

fn apply(device: &NanoleafConfig, action: &Action) -> Result<()> {
    match action {
        Action::Off => power(device, false),
        Action::Colour(colour) => put(device, "/state", colour_state(colour)?),
        Action::Scene(name) => {
            put(device, "/effects", json!({"select":name}))?;
            power(device, true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn standby_invalidates_already_applied_background() {
        let mut idle = IdleLighting::default();
        idle.applied = Some(Action::Colour("#604321".into()));
        assert!(idle.observe_standby(1));
        assert_eq!(idle.applied, None);
        idle.applied = Some(Action::Colour("#604321".into()));
        assert!(!idle.observe_standby(1));
        assert!(idle.was_managed());
    }
    #[test]
    fn standby_and_screen_off_override_scene() {
        assert_eq!(desired(true, false, Some(false), false, "Scene", "#ffffff"), Some(Action::Off));
        assert_eq!(desired(true, false, Some(true), true, "Scene", "#ffffff"), Some(Action::Off));
        assert_eq!(desired(true, false, None, false, "Scene", "#ffffff"), None);
        assert_eq!(desired(true, false, Some(true), false, "Scene", "#ffffff"), Some(Action::Scene("Scene".into())));
        assert_eq!(desired(false, true, Some(true), false, "", "#ffffff"), Some(Action::Off));
        assert_eq!(desired(false, false, Some(true), false, "", "#ffffff"), None);
    }
    #[test]
    fn static_color_converts_and_black_powers_off() {
        assert_eq!(colour_state("#00ff00").unwrap()["hue"]["value"], 120);
        assert_eq!(colour_state("#800000").unwrap()["brightness"]["value"], 50);
        assert_eq!(colour_state("#000000").unwrap()["on"]["value"], false);
        assert!(colour_state("#z12345").is_err());
    }
}
