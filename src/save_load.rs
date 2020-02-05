use crate::components::*;
use crate::components::{SerializationHelper, SerializeMe};
use crate::map::Map;
use specs::error::NoError;
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SerializeComponents, SimpleMarker};
use std::fs::File;

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<NoError, SimpleMarker<SerializeMe>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}

pub fn save_game(ecs: &mut World) {
    let map = ecs.get_mut::<Map>().unwrap().clone();
    let save_helper = ecs
        .create_entity()
        .with(SerializationHelper { map })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    {
        let data = (ecs.entities(), ecs.read_storage::<SimpleMarker<SerializeMe>>());

        //    let writer = File::create("./savegame.ron").unwrap();
        //    let mut serializer = ron::ser::Serializer::new(writer);
        let writer = File::create("./savegame.json").unwrap();
        let mut serializer = serde_json::Serializer::pretty(writer);
        serialize_individually!(
            ecs,
            serializer,
            data,
            Position,
            Renderable,
            Player,
            Viewshed,
            Monster,
            Name,
            BlocksTile,
            CombatStats,
            SufferDamage,
            WantsToMelee,
            Item,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Confusion,
            ProvidesHealing,
            InBackpack,
            WantsToPickupItem,
            WantsToUseItem,
            WantsToDropItem,
            SerializationHelper
        );
    }
    ecs.delete_entity(save_helper).expect("Crash on cleanup");
}
