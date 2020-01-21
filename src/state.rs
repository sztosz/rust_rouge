use crate::components::{Player, Position, Renderable, Viewshed};
use crate::map::{Map, TileType};
use crate::systems::VisibilitySystem;
use rltk::{Console, GameState, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub struct State {
    pub(crate) ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut visibility_system = VisibilitySystem {};
        visibility_system.run_now(&self.ecs);
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
            }
        }
    }

    fn player_input(gs: &mut State, ctx: &mut Rltk) {
        match ctx.key {
            None => {}
            Some(key) => match key {
                VirtualKeyCode::Numpad1 => Self::try_move_player(-1, 1, &mut gs.ecs),
                VirtualKeyCode::Numpad2 => Self::try_move_player(0, 1, &mut gs.ecs),
                VirtualKeyCode::Numpad3 => Self::try_move_player(1, 1, &mut gs.ecs),
                VirtualKeyCode::Numpad4 => Self::try_move_player(-1, 0, &mut gs.ecs),
                VirtualKeyCode::Numpad6 => Self::try_move_player(1, 0, &mut gs.ecs),
                VirtualKeyCode::Numpad8 => Self::try_move_player(0, -1, &mut gs.ecs),
                VirtualKeyCode::Numpad7 => Self::try_move_player(-1, -1, &mut gs.ecs),
                VirtualKeyCode::Numpad9 => Self::try_move_player(1, -1, &mut gs.ecs),
                _ => {}
            },
        }
    }


}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        State::player_input(self, ctx);
        self.run_systems();
        let map = self.ecs.get_mut::<Map>().unwrap();
        map.draw(ctx);
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}
