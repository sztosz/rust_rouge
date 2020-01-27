use crate::components::{InBackpack, Name, Position, WantsToPickupItem};
use crate::game_log::GameLog;
use specs::prelude::*;

pub struct ItemCollectionSystem {}

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
