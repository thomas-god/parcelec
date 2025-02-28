use derive_more::{AsRef, Display, From, Into};
use petname::petname;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod connection;
pub mod repository;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, From, Display, AsRef, Into)]
#[into(String)]
#[as_ref(str)]
#[from(&str, String)]
pub struct PlayerId(String);

impl Default for PlayerId {
    fn default() -> Self {
        PlayerId(Uuid::new_v4().to_string())
    }
}

#[derive(Debug, Serialize, From, Display, Into, PartialEq, Eq, Clone)]
#[from(String, &str)]
pub struct PlayerName(String);

impl PlayerName {
    pub fn random() -> PlayerName {
        PlayerName::from(petname(3, "-").unwrap_or("default".to_string()))
    }

    pub fn parse(value: &str) -> Option<PlayerName> {
        if value.is_empty() {
            return None;
        }
        Some(PlayerName(value.to_string()))
    }
}

#[cfg(test)]
mod test {
    use crate::player::PlayerId;

    #[test]
    fn test_player_id_from_into_string() {
        assert_eq!(
            PlayerId::from(String::from("toto")).to_string(),
            String::from("toto")
        );
    }

    #[test]
    fn test_player_id_as_ref() {
        assert_eq!(PlayerId::from("toto").as_ref(), "toto");
    }
}
