use crate::components::{CombatStats, Player, Position, Viewshed, WantsToMelee};
use crate::map::Map;
use crate::state::{RunState, State};
use rltk::{console, Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let entities = ecs.entities();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.fetch::<Map>();

    for (entity, _player, pos, viewshed) in (&entities, &mut players, &mut positions, &mut viewsheds).join() {
        let destination_idx = map.xy_to_idx(pos.x + delta_x, pos.y + delta_y);

        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            match target {
                None => {}
                Some(_t) => {
                    console::log("You attack monster".to_string());
                    wants_to_melee
                        .insert(
                            entity,
                            WantsToMelee {
                                target: *potential_target,
                            },
                        )
                        .expect("Add target failed");
                }
            }
        }
        if !map.blocked[destination_idx] {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));

            viewshed.dirty = true;

            let mut player_position = ecs.write_resource::<Point>();
            player_position.x = pos.x;
            player_position.y = pos.y;
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => {
            return RunState::AwaitingInput;
        }
        Some(key) => match key {
            VirtualKeyCode::Numpad1 => try_move_player(-1, 1, &mut gs.ecs),
            VirtualKeyCode::Numpad2 | VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            VirtualKeyCode::Numpad3 => try_move_player(1, 1, &mut gs.ecs),
            VirtualKeyCode::Numpad4 | VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Numpad6 | VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Numpad7 => try_move_player(-1, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad8 | VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad9 => try_move_player(1, -1, &mut gs.ecs),
            _ => {
                return RunState::AwaitingInput;
            }
        },
    }
    RunState::PlayerTurn
}
