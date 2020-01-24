use crate::{MAP_HEIGHT, MAP_WIDTH, UI_HEIGHT};
use rltk::{Console, Rltk, RGB};
use specs::prelude::*;
use crate::components::{CombatStats, Player};

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(
        0,
        MAP_HEIGHT,
        MAP_WIDTH - 1,
        UI_HEIGHT - 1,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!(" HP: {} / {}", stats.hp, stats.max_hp);
        ctx.print_color(12, MAP_HEIGHT, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &health);
        ctx.draw_bar_horizontal(28, MAP_HEIGHT, 51, stats.hp, stats.max_hp, RGB::named(rltk::RED), RGB::named(rltk::BLACK))
    }
}
