use crate::game::Game;

mod game;
mod window;
mod types;
mod resources;
mod render;

fn main() {
    let mut game = Game::new();
    game.start();
}
