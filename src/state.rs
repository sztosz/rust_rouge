use crate::components::{Player, Position, Renderable};
use crate::map::TileType;
use crate::systems::LeftWalker;
use rltk::{Console, GameState, Rltk, VirtualKeyCode, RGB};
use specs::{Join, RunNow, World, WorldExt};
use std::cmp::{max, min};

pub struct State {
    pub(crate) ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut lw = LeftWalker {};
        lw.run_now(&self.ecs);
        self.ecs.maintain();
    }

    fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
        let mut positions = ecs.write_storage::<Position>();
        let mut players = ecs.write_storage::<Player>();

        for (_player, pos) in (&mut players, &mut positions).join() {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y))
        }
    }

    fn player_input(gs: &mut State, ctx: &mut Rltk) {
        match ctx.key {
            None => {}
            Some(key) => match key {
                VirtualKeyCode::Left => Self::try_move_player(-1, 0, &mut gs.ecs),
                VirtualKeyCode::Right => Self::try_move_player(1, 0, &mut gs.ecs),
                VirtualKeyCode::Up => Self::try_move_player(0, -1, &mut gs.ecs),
                VirtualKeyCode::Down => Self::try_move_player(0, 1, &mut gs.ecs),
                _ => {}
            },
        }
    }

    fn draw_map(map: &[TileType], ctx: &mut Rltk) {
        let mut x = 0;
        let mut y = 0;

        for tile in map.iter() {
            match tile {
                TileType::Floor => ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.5, 0.5, 0.5),
                    RGB::from_f32(0.0, 0.0, 0.0),
                    rltk::to_cp437('.'),
                ),
                TileType::Wall => ctx.set(
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
        let map = self.ecs.fetch::<Vec<TileType>>();
        Self::draw_map(&map, ctx);
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}
