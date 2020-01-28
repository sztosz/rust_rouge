use crate::components::{
    CombatStats, InBackpack, Name, Position, Potion, WantsToDrinkPotion, WantsToDropItem, WantsToPickupItem,
};
use crate::game_log::GameLog;
use specs::prelude::*;

pub struct ItemCollectionSystem {}
pub struct PotionUseSystem {}
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

impl<'a> System<'a> for PotionUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDrinkPotion>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Potion>,
        WriteStorage<'a, CombatStats>,
    );
    fn run(&mut self, system_data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_to_drink_potion, names, potions, mut combat_stats) =
            system_data;

        for (entity, drink, stats, name) in (&entities, &wants_to_drink_potion, &mut combat_stats, &names).join() {
            let potion = potions.get(drink.potion);
            match potion {
                None => {}
                Some(potion) => {
                    stats.hp = i32::min(stats.max_hp, stats.hp + potion.heal_amount);
                    if entity == *player_entity {
                        gamelog.entries.insert(
                            0,
                            format!("You drink the {}, healing {}", name.name, potion.heal_amount),
                        )
                    }
                    entities.delete(drink.potion).expect("wants_to_drink removal failed")
                }
            }
        }
        wants_to_drink_potion.clear();
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
