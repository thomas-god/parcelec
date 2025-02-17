use std::future::Future;

use crate::{
    game::game_repository::GameId,
    player::{connection::PlayerConnectionContext, PlayerId},
};

#[derive(Debug, Clone, PartialEq)]
pub enum AuthPlayerToGameError {
    NoGameFound,
    NoPlayerFound,
    NoStackFound,
    Unknown,
}

pub trait AuthPlayerToGame: Clone + Send + Sync + 'static {
    fn auth_player(
        &self,
        game_id: &GameId,
        player_id: &PlayerId,
    ) -> impl Future<Output = Result<PlayerConnectionContext, AuthPlayerToGameError>> + Send;
}
