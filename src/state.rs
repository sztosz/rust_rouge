use crate::components::{Player, Position, Renderable, Viewshed};
use crate::map::{Map, TileType};
use crate::systems::{MonsterAI, VisibilitySystem};
use rltk::{Console, GameState, Point, Rltk, VirtualKeyCode};
impl State {
    fn run_systems(&mut self) {
        let mut visibility_system = VisibilitySystem {};
        visibility_system.run_now(&self.ecs);
        let mut monster_ai = MonsterAI {};
        monster_ai.run_now(&self.ecs);
        self.ecs.maintain();
    }

    fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
        let mut positions = ecs.write_storage::<Position>();
        let mut players = ecs.write_storage::<Player>();
        let mut viewsheds = ecs.write_storage::<Viewshed>();
        let map = ecs.fetch::<Map>();

        for (_player, pos, viewshed) in (&mut players, &mut positions, &mut viewsheds).join() {
            let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
            if map.tiles[destination_idx] != TileType::Wall {
                pos.x = min(79, max(0, pos.x + delta_x));
                pos.y = min(49, max(0, pos.y + delta_y));

                viewshed.dirty = true;

                let mut player_position = ecs.write_resource::<Point>();
                player_position.x = pos.x;
                player_position.y = pos.y;
            }
        }
    }

    fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
        match ctx.key {
            None => {
                return RunState::Paused;
            }
            Some(key) => match key {
                VirtualKeyCode::Numpad1 => Self::try_move_player(-1, 1, &mut gs.ecs),
                VirtualKeyCode::Numpad2 => Self::try_move_player(0, 1, &mut gs.ecs),
                VirtualKeyCode::Numpad3 => Self::try_move_player(1, 1, &mut gs.ecs),
                VirtualKeyCode::Numpad4 => Self::try_move_player(-1, 0, &mut gs.ecs),
                VirtualKeyCode::Numpad6 => Self::try_move_player(1, 0, &mut gs.ecs),
                VirtualKeyCode::Numpad8 => Self::try_move_player(0, -1, &mut gs.ecs),
                VirtualKeyCode::Numpad7 => Self::try_move_player(-1, -1, &mut gs.ecs),
                VirtualKeyCode::Numpad9 => Self::try_move_player(1, -1, &mut gs.ecs),
                _ => {
                    return RunState::Paused;
                }
            },
        }
        RunState::Running
    }
}
use specs::prelude::*;

use std::cmp::{max, min};

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    Paused,
    Running,
}

pub struct State {
    pub ecs: World,
    pub runstate: RunState,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        if self.runstate == RunState::Running {
            self.run_systems();
            self.runstate = RunState::Paused
        } else {
            self.runstate = Self::player_input(self, ctx)
        }
        let mut map = self.ecs.fetch_mut::<Map>();

        map.draw(ctx);
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph)
            }
        }
    }
}
