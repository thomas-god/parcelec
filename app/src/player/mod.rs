use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod connection;
pub mod repository;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(String);

impl From<&str> for PlayerId {
    fn from(value: &str) -> Self {
        PlayerId(value.to_string())
    }
}

impl From<String> for PlayerId {
    fn from(value: String) -> Self {
        PlayerId(value)
    }
}

impl Default for PlayerId {
    fn default() -> Self {
        PlayerId(Uuid::new_v4().to_string())
    }
}
impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl PlayerId {
    pub fn into_string(self) -> String {
        self.0
    }
}
impl AsRef<str> for PlayerId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use crate::player::PlayerId;

    #[test]
    fn test_player_id_from_into_string() {
        assert_eq!(
            PlayerId::from(String::from("toto")).into_string(),
            String::from("toto")
        );
    }

    #[test]
    fn test_player_id_as_ref() {
        assert_eq!(PlayerId::from("toto").as_ref(), "toto");
    }
}
