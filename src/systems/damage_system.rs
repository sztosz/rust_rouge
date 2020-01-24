use crate::components::{CombatStats, SufferDamage};
use specs::prelude::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut combat_stats, mut suffer_damage) = data;
        for (mut stats, damage) in (&mut combat_stats, &suffer_damage).join() {
            stats.hp -= damage.amount
        }

        suffer_damage.clear();
    }
}
