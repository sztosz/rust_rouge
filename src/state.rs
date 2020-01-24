use crate::components::{CombatStats, Player, Position, Renderable, Viewshed, WantsToMelee};
use crate::map::Map;
use crate::systems::{
    DamageSystem, MapIndexingSystem, MeleeCombatSystem, MonsterAI, VisibilitySystem,
};
use rltk::{console, Console, GameState, Point, Rltk, VirtualKeyCode};
use specs::prelude::*;

use std::cmp::{max, min};

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
}

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut visibility_system = VisibilitySystem {};
        visibility_system.run_now(&self.ecs);
        let mut monster_ai = MonsterAI {};
        monster_ai.run_now(&self.ecs);
        let mut map_indexing_system = MapIndexingSystem {};
        map_indexing_system.run_now(&self.ecs);
        let mut melee_combat_system = MeleeCombatSystem {};
        melee_combat_system.run_now(&self.ecs);
        let mut damage_system = DamageSystem {};
        damage_system.run_now(&self.ecs);
        self.ecs.maintain();
    }

    fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
        let mut positions = ecs.write_storage::<Position>();
        let mut players = ecs.write_storage::<Player>();
        let mut viewsheds = ecs.write_storage::<Viewshed>();
        let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
        let entities = ecs.entities();
        let combat_stats = ecs.read_storage::<CombatStats>();
        let map = ecs.fetch::<Map>();

        for (entity, _player, pos, viewshed) in
            (&entities, &mut players, &mut positions, &mut viewsheds).join()
        {
            let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

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

    fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
        match ctx.key {
            None => {
                return RunState::AwaitingInput;
            }
            Some(key) => match key {
                VirtualKeyCode::Numpad1 => Self::try_move_player(-1, 1, &mut gs.ecs),
                VirtualKeyCode::Numpad2 | VirtualKeyCode::Down => {
                    Self::try_move_player(0, 1, &mut gs.ecs)
                }
                VirtualKeyCode::Numpad3 => Self::try_move_player(1, 1, &mut gs.ecs),
                VirtualKeyCode::Numpad4 | VirtualKeyCode::Left => {
                    Self::try_move_player(-1, 0, &mut gs.ecs)
                }
                VirtualKeyCode::Numpad6 | VirtualKeyCode::Right => {
                    Self::try_move_player(1, 0, &mut gs.ecs)
                }
                VirtualKeyCode::Numpad7 => Self::try_move_player(-1, -1, &mut gs.ecs),
                VirtualKeyCode::Numpad8 | VirtualKeyCode::Up => {
                    Self::try_move_player(0, -1, &mut gs.ecs)
                }
                VirtualKeyCode::Numpad9 => Self::try_move_player(1, -1, &mut gs.ecs),
                _ => {
                    return RunState::AwaitingInput;
                }
            },
        }
        RunState::PlayerTurn
    }

    fn remove_the_dead(&mut self) {
        let mut dead = Vec::new();
        {
            let combat_stats = self.ecs.read_storage::<CombatStats>();
            let players = self.ecs.read_storage::<Player>();
            let entities = self.ecs.entities();
            for (entity, stats) in (&entities, &combat_stats).join() {
                if stats.hp < 1 {
                    let player = players.get(entity);
                    match player {
                        None => dead.push(entity),
                        Some(_) => console::log("You are dead!"),
                    }
                }
            }
        }
        for victim in dead {
            self.ecs
                .delete_entity(victim)
                .expect("Could not delete dead entity");
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = Self::player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        Self::remove_the_dead(self);
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
