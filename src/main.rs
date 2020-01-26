rltk::add_wasm_support!();
use rltk::{Point, Rltk};
use specs::prelude::*;

mod components;
mod game_log;
mod gui;
mod map;
mod player;
mod rect;
mod spawner;
mod state;
mod systems;

use crate::components::{
    BlocksTile, CombatStats, Monster, Name, Player, Position, Renderable, SufferDamage, Viewshed,
    WantsToMelee,
};
use crate::map::Map;
use crate::state::{RunState, State};

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 50;
const UI_HEIGHT: i32 = 10;

#[macro_use]
extern crate specs_derive;

fn main() {
    let mut context = Rltk::init_simple8x8(
        MAP_WIDTH as u32,
        (MAP_HEIGHT + UI_HEIGHT) as u32,
        "RLTK Rouge",
        "resources",
    );
    context.with_post_scanlines(true);
    let mut state = State { ecs: World::new() };
    state.ecs.register::<Position>();
    state.ecs.register::<Renderable>();
    state.ecs.register::<Player>();
    state.ecs.register::<Viewshed>();
    state.ecs.register::<Monster>();
    state.ecs.register::<Name>();
    state.ecs.register::<BlocksTile>();
    state.ecs.register::<CombatStats>();
    state.ecs.register::<WantsToMelee>();
    state.ecs.register::<SufferDamage>();

    state.ecs.insert(RunState::PreRun);
    state.ecs.insert(game_log::GameLog {
        entries: vec!["Welcome!".to_string()],
    });
    state.ecs.insert(rltk::RandomNumberGenerator::new());

    let mut map = Map::new_map_with_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    for room in map.rooms.iter_mut().skip(1) {
        let (x, y) = room.center();

        spawner::random_monster(&mut state.ecs, x, y);
    }

    state.ecs.insert(map);

    let player_entity = spawner::player(&mut state.ecs, player_x, player_y);

    state.ecs.insert(player_entity);
    state.ecs.insert(Point::new(player_x, player_y));

    rltk::main_loop(context, state)
}
