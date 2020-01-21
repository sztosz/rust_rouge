rltk::add_wasm_support!();
use rltk::{Rltk, RGB};
use specs::prelude::*;

mod components;
mod map;
mod rect;
mod state;
mod systems;

use crate::components::{Player, Position, Renderable, Viewshed};
use crate::map::Map;
use crate::state::State;

const WIDTH: i32 = 80;
const HEIGHT: i32 = 50;

#[macro_use]
extern crate specs_derive;

fn main() {
    let context = Rltk::init_simple8x8(WIDTH as u32, HEIGHT as u32, "RLTK Rouge", "resources");
    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();

    let map = Map::new_map_with_rooms_and_corridors(WIDTH, HEIGHT);
    let (player_x, player_y) = map.rooms[0].center();

    gs.ecs.insert(map);

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
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .build();

    rltk::main_loop(context, gs)
}
