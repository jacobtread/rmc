extern crate core;
extern crate core;

use crate::game::Game;

mod game;
mod math;
mod render;
mod resources;
mod types;
mod window;

fn main() {
    let mut game = Game::new();
    game.start();
}
