use crate::components::{BlocksTile, Position};
use crate::map::Map;
use specs::prelude::*;

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, positions, blockers, entities) = data;
        map.populate_blocked();
        map.clear_content_index();
        for (entity, position) in (&entities, &positions).join() {
            let idx = map.xy_idx(position.x, position.y);
            let p: Option<&BlocksTile> = blockers.get(entity);
            if let Some(_p) = p {
                map.blocked[idx] = true;
            }
            map.tile_content[idx].push(entity);
        }
    }
}
