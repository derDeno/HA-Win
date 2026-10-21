use serde_json::{json, Value};

pub const ACTIONS: [(&str, &str, &str); 3] = [
    ("restart", "Restart PC", "mdi:restart"),
    ("shutdown", "Shutdown PC", "mdi:power"),
    ("standby", "Standby PC", "mdi:sleep"),
];

pub fn base(id: &str) -> String {
    format!("ha-win/{id}")
}
pub fn status(id: &str) -> String {
    format!("{}/status", base(id))
}
pub fn notify(id: &str) -> String {
    format!("{}/notify", base(id))
}

pub fn configs(id: &str, machine: &str) -> Vec<(String, String)> {
    let device = json!({
        "identifiers": [id], "name": machine, "manufacturer": "DNO",
        "model": "HA-Win", "sw_version": env!("CARGO_PKG_VERSION")
    });
    let mut configs = Vec::new();
    for (action, name, icon) in ACTIONS {
        configs.push((
            format!("homeassistant/button/{id}/{action}/config"),
            json!({
                "name": name, "unique_id": format!("{id}_{action}"),
                "command_topic": format!("{}/{action}/set", base(id)), "payload_press": "PRESS",
                "availability_topic": status(id), "payload_available": "online",
                "payload_not_available": "offline", "icon": icon, "device": device
            })
            .to_string(),
        ));
    }
    configs.push((format!("homeassistant/notify/{id}/config"), json!({
        "name": "Notification", "unique_id": format!("{id}_notify"), "command_topic": notify(id),
        "availability_topic": status(id), "payload_available": "online",
        "payload_not_available": "offline", "device": device
    }).to_string()));
    configs
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Action(&'static str),
    Notification { title: String, message: String },
}

pub fn command(id: &str, topic: &str, payload: &[u8], retained: bool) -> Option<Command> {
    // Retained commands must never replay a shutdown or old notification after reconnect.
    if retained {
        return None;
    }
    let text = std::str::from_utf8(payload).ok()?;
    if topic == notify(id) {
        let value = serde_json::from_str::<Value>(text).ok();
        let title = value
            .as_ref()
            .and_then(|v| v.get("title"))
            .and_then(Value::as_str)
            .unwrap_or("Home Assistant");
        let message = value
            .as_ref()
            .and_then(|v| v.get("message"))
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(text);
        return Some(Command::Notification {
            title: title.into(),
            message: message.into(),
        });
    }
    if text == "PRESS" {
        for (action, _, _) in ACTIONS {
            if topic == format!("{}/{action}/set", base(id)) {
                return Some(Command::Action(action));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_matches_existing_entity_ids_and_topics() {
        let configs = configs("office_pc", "Office-PC");
        assert_eq!(configs.len(), 4);
        for (topic, payload) in configs {
            assert!(topic.starts_with("homeassistant/"));
            let v: Value = serde_json::from_str(&payload).unwrap();
            assert_eq!(v["device"]["identifiers"][0], "office_pc");
            assert_eq!(v["availability_topic"], "ha-win/office_pc/status");
        }
    }

    #[test]
    fn only_exact_live_action_commands_are_accepted() {
        assert_eq!(
            command("pc", "ha-win/pc/restart/set", b"PRESS", false),
            Some(Command::Action("restart"))
        );
        for (topic, payload, retained) in [
            ("ha-win/pc/restart/set", &b"PRESS"[..], true),
            ("ha-win/other/restart/set", &b"PRESS"[..], false),
            ("ha-win/pc/sub/restart/set", &b"PRESS"[..], false),
            ("ha-win/pc/restart/set", &b"press"[..], false),
        ] {
            assert_eq!(command("pc", topic, payload, retained), None);
        }
    }

    #[test]
    fn notifications_accept_text_json_and_non_string_fields() {
        for payload in ["Hallo", r#"{"title":42,"message":true}"#] {
            assert_eq!(
                command("pc", &notify("pc"), payload.as_bytes(), false),
                Some(Command::Notification {
                    title: "Home Assistant".into(),
                    message: payload.into()
                })
            );
        }
        assert_eq!(
            command(
                "pc",
                &notify("pc"),
                br#"{"title":"Door","message":"Open"}"#,
                false
            ),
            Some(Command::Notification {
                title: "Door".into(),
                message: "Open".into()
            })
        );
    }
}
