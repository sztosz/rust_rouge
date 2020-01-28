use crate::components::{CombatStats, Name, Player, Position, Renderable, WantsToDrinkPotion, WantsToDropItem};
use crate::game_log::GameLog;
use crate::map::Map;
use crate::systems::{
    DamageSystem, ItemCollectionSystem, ItemDropSystem, ItemUseSystem, MapIndexingSystem, MeleeCombatSystem, MonsterAI,
    VisibilitySystem,
};
use crate::{gui, player};
use rltk::{Console, GameState, Rltk};
use specs::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
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
        let mut item_collection_system = ItemCollectionSystem {};
        item_collection_system.run_now(&self.ecs);
        let mut item_use_system = ItemUseSystem {};
        item_use_system.run_now(&self.ecs);
        let mut item_drop_items = ItemDropSystem {};
        item_drop_items.run_now(&self.ecs);
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
        Self::remove_the_dead(self);

        ctx.cls();

        {
            let map = self.ecs.fetch::<Map>();
            map.draw(ctx);

            let positions = self.ecs.read_storage::<Position>();
            let renderables = self.ecs.read_storage::<Renderable>();
            let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
            data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
            for (pos, render) in data.iter() {
                let idx = map.xy_to_idx(pos.x, pos.y);
                if map.visible_tiles[idx] {
                    ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph)
                }
            }
        }

        gui::draw_ui(&self.ecs, ctx);

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
            RunState::ShowInventory => {
                let (result, entity) = gui::draw_show_inventory_item_menu(self, ctx);
                match result {
                    gui::ItemMenuResult::Cancel => RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => RunState::ShowInventory,
                    gui::ItemMenuResult::Selected => {
                        let potion = entity.unwrap();
                        let mut wants_to_drink_potion = self.ecs.write_storage::<WantsToDrinkPotion>();
                        wants_to_drink_potion
                            .insert(*self.ecs.fetch::<Entity>(), WantsToDrinkPotion { potion })
                            .expect("Unable to insert potion to wants_to_use_potion");
                        RunState::PlayerTurn
                    }
                }
            }
            RunState::ShowDropItem => {
                let (result, entity) = gui::draw_drop_item_menu(self, ctx);
                match result {
                    gui::ItemMenuResult::Cancel => RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => RunState::ShowDropItem,
                    gui::ItemMenuResult::Selected => {
                        let item = entity.unwrap();
                        let mut wants_to_drop_potion = self.ecs.write_storage::<WantsToDropItem>();
                        wants_to_drop_potion
                            .insert(*self.ecs.fetch::<Entity>(), WantsToDropItem { item })
                            .expect("Unable to insert potion to wants_to_drop_potion");
                        RunState::PlayerTurn
                    }
                }
            }
        };

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
    }
}
