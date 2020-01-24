use crate::components::{CombatStats, Name, SufferDamage, WantsToMelee};
use rltk::console;
use specs::prelude::*;
use std::cmp::max;

pub struct MeleeCombatSystem {}

static DEBUG_NAME: &str = "DEBUG: MISSING NAME";

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, wants_to_melee, names, combat_stats, mut inflict_damage) = data;

        for (_entity, wants_to_melee, name, attacker_stats) in
            (&entities, &wants_to_melee, &names, &combat_stats).join()
        {
            if attacker_stats.hp > 0 {
                let target_stats = combat_stats.get(wants_to_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_to_melee.target);

                    let damage = max(0, attacker_stats.power - target_stats.defense);
                    let victim_name = match target_name {
                        Some(name) => &name.name,
                        None => DEBUG_NAME,
                    };
                    if damage == 0 {
                        console::log(&format!("{} is unable to hurt {}", &name.name, victim_name));
                    } else {
                        console::log(&format!(
                            "{} hits {}, for {} hp.",
                            &name.name, victim_name, damage
                        ));
                        inflict_damage
                            .insert(wants_to_melee.target, SufferDamage { amount: damage })
                            .expect("Could not inflict damage");
                    }
                }
            }
        }
    }
}
