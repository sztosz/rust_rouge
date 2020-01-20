rltk::add_wasm_support!();
use rltk::{Rltk, RGB};
use specs::prelude::*;

mod components;
mod map;
mod rect;
mod state;
mod systems;

use crate::components::{Player, Position, Renderable};
use crate::state::State;

#[macro_use]
extern crate specs_derive;

fn main() {
    let context = Rltk::init_simple8x8(80, 50, "RLTK Rouge", "resources");
    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();

    let (rooms, map) = map::new_map_rooms_and_corridors();
    gs.ecs.insert(map);

    let (player_x, player_y) = rooms[0].center();

    gs.ecs
        .create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .build();

    rltk::main_loop(context, gs)
}
