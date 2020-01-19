rltk::add_wasm_support!();
use rltk::{Rltk, RGB};
use specs::prelude::*;

mod components;
mod map;
mod state;
mod systems;

use crate::components::{LeftMover, Player, Position, Renderable};
use crate::state::State;

#[macro_use]
extern crate specs_derive;

fn main() {
    let context = Rltk::init_simple8x8(80, 50, "RLTK Rouge", "resources");
    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Player>();
    gs.ecs.insert(map::new_map());
    gs.ecs
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .build();
    for i in 0..10 {
        gs.ecs
            .create_entity()
            .with(Position { x: i * 7, y: 20 })
            .with(Renderable {
                glyph: rltk::to_cp437('☺'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(LeftMover {})
            .build();
    }
    rltk::main_loop(context, gs)
}
