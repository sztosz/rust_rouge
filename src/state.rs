use crate::components::{Player, Position, Renderable, Viewshed};
use crate::map;
use crate::map::Map;
use crate::visibility_system::VisibilitySystem;
use rltk::{Console, GameState, Rltk, VirtualKeyCode, RGB};
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
            if map.tiles[destination_idx] != map::TileType::Wall {
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

    fn draw_map(ecs: &World, ctx: &mut Rltk) {
        let map = ecs.fetch::<Map>();

        let mut x = 0;
        let mut y = 0;

        for (idx, tile) in map.tiles.iter().enumerate() {
            if map.revealed_tiles[idx] {
                let glyph;
                let mut fg;
                match tile {
                    map::TileType::Floor => {
                        fg = RGB::from_f32(0.0, 0.5, 0.5);
                        glyph = rltk::to_cp437('.');
                    }

                    map::TileType::Wall => {
                        fg = RGB::from_f32(0.0, 1.0, 0.0);
                        glyph = rltk::to_cp437('#');
                    }
                }
                if !map.visible_tiles[idx] {
                    fg = fg.to_greyscale()
                }
                ctx.set(x, y, fg, RGB::from_f32(0.0, 0.0, 0.0), glyph);
            }

            x += 1;
            if x > 79 {
                x = 0;
                y += 1;
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        State::player_input(self, ctx);
        self.run_systems();
        Self::draw_map(&self.ecs, ctx);
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}
