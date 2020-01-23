use crate::components::{Monster, Name, Viewshed};
use rltk::{console, Point};
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadExpect<'a, Point>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_pos, viewsheds, monsters, names) = data;

        for (viewshed, _monster, name) in (&viewsheds, &monsters, &names).join() {
            if viewshed.visible_tiles.contains(&*player_pos) {
                console::log(&format!(
                    "{} roars: 'I am not a number! I am a monster!'",
                    name.name
                ));
            }
        }
    }
}
