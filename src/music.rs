use crate::color::RgbColor;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::{process::Command, sync::watch, task::JoinHandle};
use tokio_util::sync::CancellationToken;

const AUDIO_TIMEOUT: Duration = Duration::from_millis(500);
const MIN_BEAT_INTERVAL: Duration = Duration::from_millis(350);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Output {
    pub sensitivity: u8,
    pub intensity: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MusicSettings {
    pub enabled: bool,
    pub strip: Output,
    pub lamp: Output,
    #[serde(default = "default_top_intensity")]
    pub top_intensity: u8,
}

fn default_top_intensity() -> u8 {
    100
}

impl Default for MusicSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            top_intensity: default_top_intensity(),
            strip: Output {
                sensitivity: 55,
                intensity: 40,
            },
            lamp: Output {
                sensitivity: 60,
                intensity: 55,
            },
        }
    }
}

impl MusicSettings {
    pub fn valid(self) -> bool {
        self.top_intensity <= 100
            && [self.strip, self.lamp]
                .iter()
                .all(|output| output.sensitivity <= 100 && output.intensity <= 100)
    }
}

#[derive(Default)]
struct Pulse {
    above: bool,
    onset: Option<Instant>,
}

impl Pulse {
    fn observe(&mut self, now: Instant, level: f32, threshold: f32) {
        let above = level >= threshold;
        if above
            && !self.above
            && self
                .onset
                .is_none_or(|last| now - last >= MIN_BEAT_INTERVAL)
        {
            self.onset = Some(now);
        }
        self.above = above;
    }

    fn mix(&self, now: Instant, hold_ms: f32, fade_ms: f32, intensity: u8) -> f32 {
        self.onset.map_or(0.0, |onset| {
            let elapsed_ms = (now - onset).as_secs_f32() * 1000.0;
            let t = ((elapsed_ms - hold_ms) / fade_ms).clamp(0.0, 1.0);
            (1.0 - t * t * (3.0 - 2.0 * t)) * f32::from(intensity) / 100.0
        })
    }
}

#[derive(Default)]
struct State {
    settings: MusicSettings,
    last_sample: Option<Instant>,
    floor: Option<f32>,
    strip: Pulse,
    lamp: Pulse,
}

pub struct Music(Mutex<State>, watch::Sender<bool>);

impl Default for Music {
    fn default() -> Self {
        Self(Mutex::default(), watch::Sender::new(false))
    }
}

impl Music {
    pub fn settings(&self) -> MusicSettings {
        self.0.lock().unwrap().settings
    }

    pub fn set(&self, settings: MusicSettings) {
        let mut state = self.0.lock().unwrap();
        let changed = state.settings.enabled != settings.enabled;
        state.settings = settings;
        if changed {
            state.last_sample = None;
            state.floor = None;
            self.1.send_replace(settings.enabled);
        }
        state.strip = Pulse::default();
        state.lamp = Pulse::default();
    }

    pub fn input_active(&self) -> bool {
        self.0
            .lock()
            .unwrap()
            .last_sample
            .is_some_and(|last| last.elapsed() < AUDIO_TIMEOUT)
    }

    fn observe(&self, level: f32, now: Instant) {
        let mut state = self.0.lock().unwrap();
        if state
            .last_sample
            .is_none_or(|last| now - last >= AUDIO_TIMEOUT)
        {
            state.floor = None;
            state.strip = Pulse::default();
            state.lamp = Pulse::default();
        }
        state.last_sample = Some(now);

        let floor = state.floor.unwrap_or(level);
        let rate = if level > floor { 0.025 } else { 0.085 };
        state.floor = Some(floor + rate * (level - floor));
        let settings = state.settings;
        if settings.enabled {
            let threshold = |sensitivity: u8| {
                let factor = 2.8 - f32::from(sensitivity) * 0.017;
                (floor * factor + 0.002).max(0.004)
            };
            state
                .strip
                .observe(now, level, threshold(settings.strip.sensitivity));
            state
                .lamp
                .observe(now, level, threshold(settings.lamp.sensitivity));
        }
    }

    pub fn mixes(&self) -> (f32, u8, u8) {
        let state = self.0.lock().unwrap();
        let now = Instant::now();
        if !state.settings.enabled
            || state
                .last_sample
                .is_none_or(|last| now - last >= AUDIO_TIMEOUT)
        {
            return (0.0, 0, 0);
        }
        let strip = state
            .strip
            .mix(now, 70.0, 160.0, state.settings.strip.intensity);
        let lamp = state
            .lamp
            .mix(now, 170.0, 155.0, state.settings.lamp.intensity);
        let top = state
            .lamp
            .mix(now, 170.0, 155.0, state.settings.top_intensity);
        (
            strip,
            (lamp * 100.0).round() as u8,
            (top * 100.0).round() as u8,
        )
    }
}

pub fn brighten(color: RgbColor, mix: f32) -> RgbColor {
    let peak = color.r.max(color.g).max(color.b);
    if peak < 12 || mix == 0.0 {
        return color;
    }
    let scale = 1.0 + mix * (255.0 / f32::from(peak) - 1.0);
    let channel = |value| (f32::from(value) * scale).round() as u8;
    RgbColor::new(channel(color.r), channel(color.g), channel(color.b))
}

// MB1 + normalized bass RMS as u16 in network byte order. Only one configured peer.
pub async fn receive_levels(music: Arc<Music>, peer: Ipv4Addr, shutdown: CancellationToken) {
    let socket = match UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 18891)).await {
        Ok(socket) => socket,
        Err(error) => {
            tracing::warn!("Music UDP input unavailable: {error}");
            return;
        }
    };
    let mut packet = [0; 6];
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            result = socket.recv_from(&mut packet) => match result {
                Ok((5, SocketAddr::V4(source))) if *source.ip() == peer && &packet[..3] == b"MB1" => {
                    let rms = u16::from_be_bytes([packet[3], packet[4]]);
                    music.observe(f32::from(rms) / 65535.0, Instant::now());
                }
                Ok(_) => {}
                Err(error) => {
                    tracing::warn!("Music UDP receive failed: {error}");
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensitivity_is_independent_and_input_loss_clears_pulses() {
        let music = Music::default();
        music.set(MusicSettings {
            enabled: true,
            top_intensity: 100,
            strip: Output {
                sensitivity: 100,
                intensity: 100,
            },
            lamp: Output {
                sensitivity: 0,
                intensity: 100,
            },
        });
        let now = Instant::now();
        music.observe(0.03, now - Duration::from_millis(20));
        music.observe(0.05, now);
        let (strip, lamp, top) = music.mixes();
        assert_eq!(strip, 1.0);
        assert_eq!(lamp, 0);
        assert_eq!(top, 0);
        music.set(MusicSettings::default());
        assert_eq!(music.mixes(), (0.0, 0, 0));

        let music = Music::default();
        music.set(MusicSettings {
            enabled: true,
            ..MusicSettings::default()
        });
        let old = now - AUDIO_TIMEOUT;
        music.observe(0.03, old - Duration::from_millis(20));
        music.observe(0.2, old);
        assert_eq!(music.mixes(), (0.0, 0, 0));
    }

    #[test]
    fn alsa_reads_stereo_bass_and_rejects_silence() {
        let source = NativeInput::Alsa("test-pcm".into());
        let block: Vec<_> = (0..960)
            .flat_map(|index| {
                let phase = 80.0 * index as f32 * std::f32::consts::TAU / 48000.0;
                let bytes = ((phase.sin() * 8000.0) as i16).to_le_bytes();
                [bytes[0], bytes[1], bytes[0], bytes[1]]
            })
            .collect();
        assert_eq!(block.len(), source.block_size());
        assert!(source.bass_level(&mut Bass::default(), &block) > 0.10);
        assert_eq!(source.bass_level(&mut Bass::default(), &[0; 3840]), 0.0);
    }

    #[test]
    fn top_strength_defaults_to_full_and_is_independent_of_ring_strength() {
        let settings: MusicSettings = serde_json::from_value(serde_json::json!({
            "enabled": true,
            "strip": {"sensitivity": 100, "intensity": 0},
            "lamp": {"sensitivity": 100, "intensity": 0}
        }))
        .unwrap();
        assert_eq!(settings.top_intensity, 100);
        let music = Music::default();
        music.set(settings);
        let now = Instant::now();
        music.observe(0.01, now - Duration::from_millis(20));
        music.observe(0.1, now);
        assert_eq!(music.mixes(), (0.0, 0, 100));
        assert!(!MusicSettings {
            top_intensity: 101,
            ..settings
        }
        .valid());
    }

    #[test]
    fn envelope_holds_peak_then_fades_to_the_exact_base() {
        let onset = Instant::now();
        let pulse = Pulse {
            above: true,
            onset: Some(onset),
        };
        let mix = |ms| pulse.mix(onset + Duration::from_millis(ms), 170.0, 150.0, 100);
        let base = RgbColor::new(40, 20, 0);
        assert_eq!(brighten(base, mix(170)), RgbColor::new(255, 128, 0));
        assert_eq!(brighten(base, mix(245)), RgbColor::new(148, 74, 0));
        assert_eq!(brighten(base, mix(320)), base);
    }

    #[test]
    fn boost_preserves_black_and_color_ratios() {
        assert_eq!(
            brighten(RgbColor::new(0, 0, 0), 0.35),
            RgbColor::new(0, 0, 0)
        );
        assert_eq!(
            brighten(RgbColor::new(40, 20, 0), 1.0),
            RgbColor::new(255, 128, 0)
        );
    }
}

#[derive(Default)]
pub struct LightFrame {
    pub base_colors: Vec<RgbColor>,
    pub output_colors: Vec<RgbColor>,
    pub music_base_colors: Vec<RgbColor>,
    pub lamp_mix: u8,
    pub top_mix: u8,
    pub sampled_at: Option<Instant>,
}

impl LightFrame {
    pub fn fresh_lamp_mixes(&self) -> (u8, u8) {
        if self
            .sampled_at
            .is_some_and(|time| time.elapsed() < Duration::from_millis(160))
        {
            (self.lamp_mix, self.top_mix)
        } else {
            (0, 0)
        }
    }
}

// Bass RMS of 20 ms mono blocks at 16 kHz, with a 35–170 Hz bandpass.
#[derive(Default)]
struct Bass {
    slow: f32,
    filtered: f32,
}

impl Bass {
    fn rms(&mut self, samples: impl Iterator<Item = f32>) -> f32 {
        let mut energy = 0.0;
        for sample in samples {
            self.slow += 0.01365 * (sample - self.slow);
            self.filtered += 0.06458 * (sample - self.slow - self.filtered);
            energy += self.filtered * self.filtered;
        }
        (energy / 320.0).sqrt()
    }
}

enum NativeInput {
    Pulse(String),
    Alsa(String),
}

impl NativeInput {
    fn command(&self) -> Command {
        match self {
            Self::Pulse(source) => {
                let mut command = Command::new("parec");
                command.arg(format!("--device={source}")).args([
                    "--raw",
                    "--format=s16le",
                    "--rate=16000",
                    "--channels=1",
                    "--latency-msec=25",
                ]);
                command
            }
            Self::Alsa(device) => {
                let mut command = Command::new("arecord");
                command.args([
                    "-q",
                    "-D",
                    device,
                    "-t",
                    "raw",
                    "-c",
                    "2",
                    "-f",
                    "S16_LE",
                    "-r",
                    "48000",
                    "--period-time=20000",
                    "--buffer-time=80000",
                ]);
                command
            }
        }
    }

    fn block_size(&self) -> usize {
        match self {
            Self::Pulse(_) => 640,
            Self::Alsa(_) => 3840,
        }
    }

    fn bass_level(&self, bass: &mut Bass, block: &[u8]) -> f32 {
        let pcm = |pair| f32::from(i16::from_le_bytes(pair)) / 32768.0;
        match self {
            Self::Pulse(_) => bass.rms(block.as_chunks::<2>().0.iter().copied().map(pcm)),
            Self::Alsa(_) => {
                // Average three consecutive stereo frames to make one mono 16 kHz sample.
                bass.rms(block.as_chunks::<12>().0.iter().map(|frame| {
                    frame
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .copied()
                        .map(pcm)
                        .sum::<f32>()
                        / 6.0
                }))
            }
        }
    }
}

pub fn start_input(
    path: &Path,
    music: Arc<Music>,
    shutdown: CancellationToken,
) -> Option<JoinHandle<()>> {
    let configured = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => {
            tracing::warn!("Cannot read music audio source: {error}");
            String::new()
        }
    };
    let configured = configured.trim();
    if let Some(source) = std::env::var("LG_HUE_MUSIC_SOURCE")
        .ok()
        .or_else(|| configured.strip_prefix("pulse:").map(str::to_owned))
        .filter(|source| !source.is_empty())
    {
        return Some(tokio::spawn(capture_native(
            music,
            NativeInput::Pulse(source),
            shutdown,
        )));
    }
    if let Some(device) = configured
        .strip_prefix("alsa:")
        .filter(|device| !device.is_empty())
    {
        return Some(tokio::spawn(capture_native(
            music,
            NativeInput::Alsa(device.to_owned()),
            shutdown,
        )));
    }
    let peer = std::env::var("LG_HUE_MUSIC_PEER")
        .ok()
        .or_else(|| configured.strip_prefix("udp:").map(str::to_owned));
    if let Some(peer) = peer {
        match peer.parse() {
            Ok(peer) => return Some(tokio::spawn(receive_levels(music, peer, shutdown))),
            Err(_) => tracing::warn!("Music UDP peer must be an IPv4 address"),
        }
    } else if !configured.is_empty() {
        tracing::warn!("Music input must use alsa:DEVICE, pulse:SOURCE or udp:IP");
    }
    None
}

async fn capture_native(music: Arc<Music>, source: NativeInput, shutdown: CancellationToken) {
    use std::process::Stdio;
    use tokio::io::AsyncReadExt;

    let mut enabled = music.1.subscribe();
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            result = enabled.wait_for(|value| *value) => if result.is_err() { break },
        }
        let child = source
            .command()
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .spawn();
        match child {
            Ok(mut child) => {
                let mut audio = child.stdout.take().expect("parec stdout is piped");
                let mut bass = Bass::default();
                let mut block = vec![0; source.block_size()];
                loop {
                    tokio::select! {
                        _ = shutdown.cancelled() => return,
                        _ = enabled.changed() => break,
                        result = audio.read_exact(&mut block) => match result {
                            Ok(_) => music.observe(source.bass_level(&mut bass, &block), Instant::now()),
                            Err(_) => break,
                        }
                    }
                }
                let _ = child.kill().await;
                if !*enabled.borrow() {
                    continue;
                }
                tracing::warn!("Music monitor stream ended; retrying");
            }
            Err(error) => tracing::warn!("Unable to start music monitor: {error}"),
        }
        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = enabled.changed() => {},
            _ = tokio::time::sleep(Duration::from_secs(4)) => {}
        }
    }
}
