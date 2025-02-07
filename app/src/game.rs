pub enum Game {
    Open,
    Running,
    Ended,
}

impl Default for Game {
    fn default() -> Self {
        Self::Open
    }
}
pub struct Player;
pub enum GameEvent {
    RegisterPlayer(Player),
    StartGame(),
}

impl Game {
    pub fn handle_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::RegisterPlayer(player) => {}
            GameEvent::StartGame() => *self = Game::Running,
        }
    }
}
