use crate::components::{Player, Position, Renderable};
use crate::map;
use rltk::{Console, GameState, Rltk, VirtualKeyCode, RGB};
use specs::{Join, World, WorldExt};
use std::cmp::{max, min};

pub struct State {
    pub(crate) ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        self.ecs.maintain();
    }

    fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
        let mut positions = ecs.write_storage::<Position>();
        let mut players = ecs.write_storage::<Player>();
        let map = ecs.fetch::<Vec<map::TileType>>();

        for (_player, pos) in (&mut players, &mut positions).join() {
            let destination_idx = map::xy_idx(pos.x + delta_x, pos.y + delta_y);
            if map[destination_idx] != map::TileType::Wall {
                pos.x = min(79, max(0, pos.x + delta_x));
                pos.y = min(49, max(0, pos.y + delta_y))
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

    fn draw_map(map: &[map::TileType], ctx: &mut Rltk) {
        let mut x = 0;
        let mut y = 0;

        for tile in map.iter() {
            match tile {
                map::TileType::Floor => ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.5, 0.5, 0.5),
                    RGB::from_f32(0.0, 0.0, 0.0),
                    rltk::to_cp437('.'),
                ),
                map::TileType::Wall => ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.0, 1.0, 0.0),
                    RGB::from_f32(0.0, 0.0, 0.0),
                    rltk::to_cp437('#'),
                ),
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
        self.run_systems();
        State::player_input(self, ctx);
        let map = self.ecs.fetch::<Vec<map::TileType>>();
        Self::draw_map(&map, ctx);
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}
