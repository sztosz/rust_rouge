use crate::components::{
    AreaOfEffect, CombatStats, Confusion, Consumable, InBackpack, InflictsDamage, Name, Position, ProvidesHealing,
    SufferDamage, WantsToDropItem, WantsToPickupItem, WantsToUseItem,
};
use crate::game_log::GameLog;
use crate::map::Map;
use specs::prelude::*;

pub struct ItemCollectionSystem {}
pub struct ItemUseSystem {}
pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_to_pickup, mut positions, names, mut in_backpack) = data;

        for pickup in wants_to_pickup.join() {
            positions.remove(pickup.item);
            in_backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("Unable to insert item to backpack");

            if pickup.collected_by == *player_entity {
                gamelog
                    .entries
                    .insert(0, format!("You pick up {}", names.get(pickup.item).unwrap().name));
            }
        }

        wants_to_pickup.clear();
    }
}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        WriteStorage<'a, AreaOfEffect>,
        WriteStorage<'a, Confusion>,
    );
    fn run(&mut self, system_data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            map,
            entities,
            mut wants_to_use_item,
            names,
            consumables,
            provides_healing,
            inflicts_damage,
            mut combat_stats,
            mut suffer_damage,
            area_of_effect,
            mut confusion,
        ) = system_data;

        for (entity, item_use, item_name) in (&entities, &wants_to_use_item, &names).join() {
            let mut used = false;
            let mut action_targets: Vec<Entity> = Vec::new();

            match item_use.target {
                None => action_targets.push(*player_entity),
                Some(target) => match area_of_effect.get(item_use.item) {
                    None => {
                        let idx = map.point_to_idx(target);
                        for mob in map.tile_content[idx].iter() {
                            action_targets.push(*mob);
                        }
                    }
                    Some(area_of_effect) => {
                        let mut blast = rltk::field_of_view(target, area_of_effect.radius, &*map);
                        blast.retain(|p| p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1);
                        for point in blast.iter() {
                            let idx = map.point_to_idx(*point);
                            for mob in map.tile_content[idx].iter() {
                                action_targets.push(*mob)
                            }
                        }
                    }
                },
            }
            match provides_healing.get(item_use.item) {
                None => {}
                Some(healing) => {
                    for target in action_targets.iter() {
                        if let Some(stats) = combat_stats.get_mut(*target) {
                            stats.hp = i32::min(stats.max_hp, stats.hp + healing.heal_amount);
                            if entity == *player_entity {
                                gamelog.entries.insert(
                                    0,
                                    format!("You use the {}, healing {}", item_name.name, healing.heal_amount),
                                )
                            }
                        }
                    }
                    used = true;
                }
            }
            match inflicts_damage.get(item_use.item) {
                None => {}
                Some(damage) => {
                    for target in action_targets.iter() {
                        suffer_damage
                            .insert(*target, SufferDamage { amount: damage.damage })
                            .expect("Unable to insert damage to mob entity");
                        if entity == *player_entity {
                            let target_name = names.get(*target).unwrap();
                            gamelog.entries.insert(
                                0,
                                format!(
                                    "You use {} on {}, inflicting {} hp.",
                                    item_name.name, target_name.name, damage.damage
                                ),
                            );
                        }
                    }
                    used = true;
                }
            }
            let mut add_confusion = Vec::new();
            match confusion.get(item_use.item) {
                None => {}
                Some(confuses) => {
                    for target in action_targets.iter() {
                        add_confusion.push((*target, confuses.turns));
                        if entity == *player_entity {
                            let target_name = names.get(*target).unwrap();
                            gamelog.entries.insert(
                                0,
                                format!("You use {} on {}, confusing it.", item_name.name, target_name.name),
                            );
                        }
                    }
                    used = true;
                }
            }
            for (entity, turns) in add_confusion.iter() {
                confusion
                    .insert(*entity, Confusion { turns: *turns })
                    .expect("Unable to insert confusion status");
            }
            if used {
                match consumables.get(item_use.item) {
                    None => {}
                    Some(_) => entities
                        .delete(item_use.item)
                        .expect("Entity removal for consumables failed"),
                }
            }
        }
        wants_to_use_item.clear();
    }
}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );
    fn run(&mut self, system_data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_to_drop_item, names, mut positions, mut in_backpack) =
            system_data;

        for (entity, to_drop, name) in (&entities, &wants_to_drop_item, &names).join() {
            let position = *positions.get(entity).unwrap();

            positions
                .insert(to_drop.item, position)
                .expect("Unable to insert dropped item position");

            in_backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.insert(0, format!("You dropped the {}.", name.name));
            }
        }
        wants_to_drop_item.clear();
    }
}
