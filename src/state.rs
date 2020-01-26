use crate::components::{CombatStats, Name, Player, Position, Renderable};
use crate::game_log::GameLog;
use crate::map::Map;
use crate::systems::{DamageSystem, MapIndexingSystem, MeleeCombatSystem, MonsterAI, VisibilitySystem};
use crate::{gui, player};
use rltk::{Console, GameState, Rltk};
use specs::prelude::*;

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

    fn remove_the_dead(&mut self) {
        let mut dead = Vec::new();
        {
            let combat_stats = self.ecs.read_storage::<CombatStats>();
            let players = self.ecs.read_storage::<Player>();
            let entities = self.ecs.entities();
            let names = self.ecs.read_storage::<Name>();
            let mut log = self.ecs.write_resource::<GameLog>();
            for (entity, stats) in (&entities, &combat_stats).join() {
                if stats.hp < 1 {
                    let player = players.get(entity);
                    match player {
                        None => {
                            let victim_name = names.get(entity);
                            log.entries
                                .insert(0, format!("{} is dead", &victim_name.expect("Missing name").name));
                            dead.push(entity)
                        }
                        Some(_) => log.entries.insert(0, "You are dead".to_string()),
                    }
                }
            }
        }
        for victim in dead {
            self.ecs.delete_entity(victim).expect("Could not delete dead entity");
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let runstate = *self.ecs.fetch::<RunState>();

        let newrunstate = match runstate {
            RunState::PreRun => {
                self.run_systems();
                RunState::AwaitingInput
            }
            RunState::AwaitingInput => player::player_input(self, ctx),
            RunState::PlayerTurn => {
                self.run_systems();
                RunState::MonsterTurn
            }
            RunState::MonsterTurn => {
                self.run_systems();
                RunState::AwaitingInput
            }
        };

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        Self::remove_the_dead(self);
        let map = self.ecs.fetch::<Map>();

        map.draw(ctx);
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph)
            }
        }

        gui::draw_ui(&self.ecs, ctx);
    }
}
