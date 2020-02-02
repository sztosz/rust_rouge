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

#[derive(SystemData)]
pub struct ItemUseSystemData<'a> {
    player_entity: ReadExpect<'a, Entity>,
    gamelog: WriteExpect<'a, GameLog>,
    map: ReadExpect<'a, Map>,
    entities: Entities<'a>,
    wants_to_use_item: WriteStorage<'a, WantsToUseItem>,
    names: ReadStorage<'a, Name>,
    consumables: ReadStorage<'a, Consumable>,
    provides_healing: ReadStorage<'a, ProvidesHealing>,
    inflicts_damage: ReadStorage<'a, InflictsDamage>,
    combat_stats: WriteStorage<'a, CombatStats>,
    suffer_damage: WriteStorage<'a, SufferDamage>,
    area_of_effect: WriteStorage<'a, AreaOfEffect>,
    confusion: WriteStorage<'a, Confusion>,
}

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

impl ItemUseSystem {
    fn targets(used_item: &WantsToUseItem, data: &<ItemUseSystem as System>::SystemData) -> Vec<Entity> {
        let mut action_targets: Vec<Entity> = Vec::new();

        match used_item.target {
            None => action_targets.push(*data.player_entity),
            Some(target) => match data.area_of_effect.get(used_item.item) {
                None => {
                    let idx = data.map.point_to_idx(target);
                    for mob in data.map.tile_content[idx].iter() {
                        action_targets.push(*mob);
                    }
                }
                Some(area_of_effect) => {
                    let mut blast = rltk::field_of_view(target, area_of_effect.radius, &*data.map);
                    blast.retain(|p| p.x > 0 && p.x < data.map.width - 1 && p.y > 0 && p.y < data.map.height - 1);
                    for point in blast.iter() {
                        let idx = data.map.point_to_idx(*point);
                        for mob in data.map.tile_content[idx].iter() {
                            action_targets.push(*mob)
                        }
                    }
                }
            },
        }
        action_targets
    }
}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = ItemUseSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (entity, used_item, item_name) in (&data.entities, &data.wants_to_use_item, &data.names).join() {
            let mut used = false;
            let action_targets = Self::targets(used_item, &data);

            // TODO: SPLIT IT TO SEPARATE SYUSTEMS
            match data.provides_healing.get(used_item.item) {
                None => {}
                Some(healing) => {
                    for target in action_targets.iter() {
                        if let Some(stats) = data.combat_stats.get_mut(*target) {
                            stats.hp = i32::min(stats.max_hp, stats.hp + healing.heal_amount);
                            if entity == *data.player_entity {
                                data.gamelog.entries.insert(
                                    0,
                                    format!("You use the {}, healing {}", item_name.name, healing.heal_amount),
                                )
                            }
                        }
                    }
                    used = true;
                }
            }
            match data.inflicts_damage.get(used_item.item) {
                None => {}
                Some(damage) => {
                    for target in action_targets.iter() {
                        data.suffer_damage
                            .insert(*target, SufferDamage { amount: damage.damage })
                            .expect("Unable to insert damage to target entity");
                        if entity == *data.player_entity {
                            let target_name = data.names.get(*target).unwrap();
                            data.gamelog.entries.insert(
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
            match data.confusion.get(used_item.item) {
                None => {}
                Some(confuses) => {
                    for target in action_targets.iter() {
                        add_confusion.push((*target, confuses.turns));
                        if entity == *data.player_entity {
                            let target_name = data.names.get(*target).unwrap();
                            data.gamelog.entries.insert(
                                0,
                                format!("You use {} on {}, confusing it.", item_name.name, target_name.name),
                            );
                        }
                    }
                    used = true;
                }
            }
            for (entity, turns) in add_confusion.iter() {
                data.confusion
                    .insert(*entity, Confusion { turns: *turns })
                    .expect("Unable to insert confusion status");
            }
            if used {
                match data.consumables.get(used_item.item) {
                    None => {}
                    Some(_) => data
                        .entities
                        .delete(used_item.item)
                        .expect("Entity removal for consumables failed"),
                }
            }
        }
        data.wants_to_use_item.clear();
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
