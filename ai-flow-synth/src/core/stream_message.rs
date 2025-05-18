use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamMessage {
    #[serde(rename = "d")]
    Delta(String),
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_serde_message() {
        let message = StreamMessage::Delta("test".to_string());
        let serialized = serde_json::to_string(&message).unwrap();
        assert_eq!(serialized, r#"{"d":"test"}"#,);
    }
}
