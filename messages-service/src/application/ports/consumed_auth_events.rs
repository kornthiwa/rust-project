//! JSON shape published by auth-service on `auth.events`.
//! When auth adds variants, add matching variants here so this consumer can deserialize.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum ConsumedAuthEvent {
    #[serde(rename = "user_registered")]
    UserRegistered {
        account_id: u64,
        username: String,
        occurred_at: String,
    },
    #[serde(rename = "user_logged_in")]
    UserLoggedIn {
        account_id: u64,
        username: String,
        occurred_at: String,
        expires_in_seconds: i64,
    },
}

#[cfg(test)]
mod tests {
    use super::ConsumedAuthEvent;

    #[test]
    fn consumed_auth_event_json_roundtrip_user_registered() {
        let event = ConsumedAuthEvent::UserRegistered {
            account_id: 9,
            username: "u".into(),
            occurred_at: "2026-05-03T12:00:00Z".into(),
        };
        let json = serde_json::to_string(&event).expect("serialize");
        let back: ConsumedAuthEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event, back);
    }
}
