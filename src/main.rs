use crate::game::Game;

mod game;
mod window;
mod rutil;
mod types;
mod resources;

fn main() {
    let mut game = Game::new();
    game.start();
}
