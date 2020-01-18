use crate::components::{Player, Position, Renderable};
use crate::systems::LeftWalker;
use rltk::{Console, GameState, Rltk, VirtualKeyCode};
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
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        self.run_systems();
        State::player_input(self, ctx);
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}
