use serde_json::Value;

const FOREGROUND_URI: &str = "luna://com.webos.applicationManager/getForegroundAppInfo";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureSource {
    Hdmi,
    LocalMedia,
    Youtube,
    Kinopoisk,
    Lampa,
    Ivi,
    Reelo,
    Okko,
    Unsupported,
    Unknown,
}

impl CaptureSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Hdmi => "HDMI",
            Self::LocalMedia => "Local media",
            Self::Youtube => "YouTube",
            Self::Kinopoisk => "Kinopoisk",
            Self::Lampa => "Lampa",
            Self::Ivi => "Ivi",
            Self::Reelo => "Reelo",
            Self::Okko => "Okko",
            Self::Unsupported => "Unsupported",
            Self::Unknown => "Unknown",
        }
    }

    pub fn capture_fps(self, hdmi_auto_fps: Option<bool>) -> Option<f64> {
        hdmi_auto_fps.map(|enabled| if enabled && self == Self::Hdmi { 60.0 } else { 30.0 })
    }

    pub fn compatible(self, enabled: [bool; 6]) -> Option<bool> {
        match self {
            Self::Hdmi | Self::LocalMedia => Some(true),
            Self::Youtube => Some(enabled[0]),
            Self::Kinopoisk => Some(enabled[1]),
            Self::Lampa => Some(enabled[2]),
            Self::Ivi => Some(enabled[3]),
            Self::Reelo => Some(enabled[4]),
            Self::Okko => Some(enabled[5]),
            Self::Unsupported => Some(false),
            Self::Unknown => None,
        }
    }
}

fn parse_source(value: &Value) -> CaptureSource {
    if value["returnValue"] != true {
        return CaptureSource::Unknown;
    }
    let Some(id) = value["appId"].as_str().filter(|id| {
        !id.is_empty()
            && id.len() <= 256
            && id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
    }) else {
        return CaptureSource::Unknown;
    };
    match id {
        "com.webos.app.hdmi1"
        | "com.webos.app.hdmi2"
        | "com.webos.app.hdmi3"
        | "com.webos.app.hdmi4" => CaptureSource::Hdmi,
        "com.webos.app.mediadiscovery" => CaptureSource::LocalMedia,
        "youtube.leanback.v4" => CaptureSource::Youtube,
        "com.685631.3411" => CaptureSource::Kinopoisk,
        "lampa.tv" => CaptureSource::Lampa,
        "ivi" => CaptureSource::Ivi,
        "ru.reelo.app" | "ru.reelo.app.rustpreview" => CaptureSource::Reelo,
        "yota.play" => CaptureSource::Okko,
        _ => CaptureSource::Unsupported,
    }
}

/// Foreground identity is a capture-policy hint, not DRM detection.
pub async fn current_source() -> CaptureSource {
    crate::luna::query(FOREGROUND_URI, "{}")
        .await
        .as_ref()
        .map(parse_source)
        .unwrap_or(CaptureSource::Unknown)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn hdmi_auto_fps_leaves_other_sources_at_thirty() {
        assert_eq!(CaptureSource::Hdmi.capture_fps(None), None);
        assert_eq!(CaptureSource::Hdmi.capture_fps(Some(false)), Some(30.0));
        assert_eq!(CaptureSource::Hdmi.capture_fps(Some(true)), Some(60.0));
        for source in [
            CaptureSource::Youtube,
            CaptureSource::LocalMedia,
            CaptureSource::Kinopoisk,
            CaptureSource::Reelo,
            CaptureSource::Okko,
        ] {
            assert_eq!(source.capture_fps(Some(true)), Some(30.0));
        }
    }

    #[test]
    fn lg_local_media_is_an_allowed_capture_source() {
        let source = parse_source(&json!({"returnValue":true,"appId":"com.webos.app.mediadiscovery"}));
        assert_eq!(source, CaptureSource::LocalMedia);
        assert_eq!(source.compatible([true; 6]), Some(true));
    }

    #[test]
    fn selected_apps_require_exact_ids() {
        for (id, expected, index) in [
            ("youtube.leanback.v4", CaptureSource::Youtube, 0),
            ("com.685631.3411", CaptureSource::Kinopoisk, 1),
            ("lampa.tv", CaptureSource::Lampa, 2),
            ("ivi", CaptureSource::Ivi, 3),
            ("ru.reelo.app", CaptureSource::Reelo, 4),
            ("ru.reelo.app.rustpreview", CaptureSource::Reelo, 4),
            ("yota.play", CaptureSource::Okko, 5),
        ] {
            let source = parse_source(&json!({"returnValue": true, "appId": id}));
            assert_eq!(source, expected);
            let mut enabled = [true; 6];
            enabled[index] = false;
            assert_eq!(source.compatible(enabled), Some(false));
            assert_eq!(source.compatible([true; 6]), Some(true));
            let other = parse_source(&json!({"returnValue": true, "appId": format!("{id}.other")}));
            assert_eq!(other.compatible([true; 6]), Some(false));
        }
    }

    #[test]
    fn foreground_identity_classifies_only_exact_hdmi_ids_and_preserves_unknowns() {
        for id in [
            "com.webos.app.hdmi1",
            "com.webos.app.hdmi2",
            "com.webos.app.hdmi3",
            "com.webos.app.hdmi4",
        ] {
            let source =
                parse_source(&json!({"returnValue":true,"appId":id,"title":"private title"}));
            assert_eq!(source, CaptureSource::Hdmi);
            assert_eq!(source.compatible([true; 6]), Some(true));
            assert_eq!(source.label(), "HDMI");
        }
        for id in [
            "netflix",
            "com.webos.app.netflix",
            "com.webos.app.home",
            "com.webos.app.browser",
            "com.webos.app.livetv",
            "com.webos.app.hdmi5",
            "com.webos.app.hdmi1.extra",
        ] {
            let source = parse_source(&json!({"returnValue":true,"appId":id}));
            assert_eq!(source, CaptureSource::Unsupported);
            assert_eq!(source.compatible([true; 6]), Some(false));
        }
        for value in [
            json!({}),
            json!({"appId":"com.webos.app.hdmi1"}),
            json!({"returnValue":false,"appId":"netflix"}),
            json!({"returnValue":true,"appId":""}),
            json!({"returnValue":true,"appId":12}),
            json!({"returnValue":true,"appId":"bad\napp"}),
            json!({"returnValue":true,"foregroundAppInfo":[{"appId":"netflix"}]}),
        ] {
            assert_eq!(parse_source(&value), CaptureSource::Unknown);
        }
        assert_eq!(
            parse_source(&json!("not an object")),
            CaptureSource::Unknown
        );
        assert_eq!(CaptureSource::Unknown.compatible([true; 6]), None);
        assert_eq!(CaptureSource::Unknown.label(), "Unknown");
    }
}
