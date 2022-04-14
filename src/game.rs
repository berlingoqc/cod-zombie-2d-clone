use crate::player::Player;

#[derive(Default)]
pub struct Game {
    pub player: Player,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Playing,
    GameOver,
}
