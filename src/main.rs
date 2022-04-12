use crate::game::Game;

mod game;
mod window;
mod texture;
mod types;

fn main() {
    let mut game = Game::new();
    game.start();
}
