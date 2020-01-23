use crate::components::{Monster, Name, Position, Viewshed};
use crate::map::Map;
use rltk::{console, Point};
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, player_pos, mut viewsheds, monsters, names, mut positions) = data;

        for (mut viewshed, _monster, name, mut pos) in
            (&mut viewsheds, &monsters, &names, &mut positions).join()
        {
            if viewshed.visible_tiles.contains(&*player_pos) {
                console::log(&format!("{} roars: 'I will get you!'", name.name));

                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y) as i32,
                    map.xy_idx(player_pos.x, player_pos.y) as i32,
                    &*map,
                );

                if path.success && path.steps.len() > 1 {
                    pos.x = path.steps[1] as i32 % map.width;
                    pos.y = path.steps[1] as i32 / map.width;
                    viewshed.dirty = true;
                }
            }
        }
    }
}
