use crate::components::{CombatStats, InBackpack, Name, Player, Position, Viewshed};
use crate::game_log::GameLog;
use crate::state::State;
use crate::{MAP_HEIGHT, MAP_WIDTH, UI_HEIGHT};
use rltk::{Console, Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}
#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}
#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

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
        ctx.print_color(
            12,
            MAP_HEIGHT,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            &health,
        );
        ctx.draw_bar_horizontal(
            28,
            MAP_HEIGHT,
            51,
            stats.hp,
            stats.max_hp,
            RGB::named(rltk::RED),
            RGB::named(rltk::BLACK),
        )
    }

    let log = ecs.fetch::<GameLog>();

    let mut y = MAP_HEIGHT + 1;
    for msg in log.entries.iter() {
        if y < MAP_HEIGHT + UI_HEIGHT {
            ctx.print(2, y, msg)
        };
        y += 1;
    }

    draw_tooltips(ecs, ctx);
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    // TODO: MAKE TOOLTIPS VISIBLE ONLY WHEN POINTING TO VISIBLE TILE
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let mouse_pos = ctx.mouse_pos();
    if mouse_pos.0 <= MAP_WIDTH || mouse_pos.1 <= MAP_HEIGHT {
        let mut tooltip = Vec::new();
        for (name, position) in (&names, &positions).join() {
            if position.x == mouse_pos.0 && position.y == mouse_pos.1 {
                tooltip.push(name.name.to_string());
            }
        }

        if !tooltip.is_empty() {
            let mut width = 0;
            for string in tooltip.iter() {
                if width < string.len() {
                    width = string.len();
                }
            }
            width += 3;

            // TODO: REMOVE ALL MAGIC NUMBERS ADN SIMPLIFY
            if mouse_pos.0 > 40 {
                let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
                let left_x = mouse_pos.0 - width as i32;
                let mut y = mouse_pos.1;
                for string in tooltip.iter() {
                    ctx.print_color(left_x, y, RGB::named(rltk::WHITE), RGB::named(rltk::GRAY), string);
                    let padding = width - string.len() - 1;
                    for i in 0..padding as i32 {
                        ctx.print_color(arrow_pos.x - i, y, RGB::named(rltk::WHITE), RGB::named(rltk::GRAY), " ");
                    }
                    y += 1
                }

                ctx.print_color(
                    arrow_pos.x,
                    arrow_pos.y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::GRAY),
                    "->",
                );
            } else {
                let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
                let left_x = mouse_pos.0 + 3;
                let mut y = mouse_pos.1;
                for string in tooltip.iter() {
                    ctx.print_color(left_x + 1, y, RGB::named(rltk::WHITE), RGB::named(rltk::GRAY), string);
                    let padding = width - string.len() - 1;
                    for i in 0..padding as i32 {
                        ctx.print_color(
                            arrow_pos.x + 1 + i,
                            y,
                            RGB::named(rltk::WHITE),
                            RGB::named(rltk::GRAY),
                            " ",
                        );
                    }
                    y += 1
                }

                ctx.print_color(
                    arrow_pos.x,
                    arrow_pos.y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::GRAY),
                    "<-",
                );
            }
        }
    }
}

pub fn draw_inventory_menu(state: &mut State, ctx: &mut Rltk, title: &str) -> (ItemMenuResult, Option<Entity>) {
    let white = RGB::named(rltk::WHITE);
    let black = RGB::named(rltk::BLACK);
    let yellow = RGB::named(rltk::YELLOW);

    let player_entity = state.ecs.fetch::<Entity>();
    let names = state.ecs.read_storage::<Name>();
    let in_backpack = state.ecs.read_storage::<InBackpack>();
    let entities = state.ecs.entities();

    let count = (&in_backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity)
        .count() as i32;

    let y = 10;
    let x = 10;

    ctx.draw_box(x, y, 31, count + 3, white, black);
    ctx.print_color(x + 5, y, yellow, black, title);
    ctx.print_color(x + 5, y + count + 3, yellow, black, "ESCAPE to cancel");

    let mut equippable = Vec::new();

    for (i, (_in_backpack, entitiy, name)) in (&in_backpack, &entities, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity)
        .enumerate()
    {
        ctx.set(x + 1, y + 2 + i as i32, white, black, rltk::to_cp437('('));
        ctx.set(x + 2, y + 2 + i as i32, yellow, black, 97 + i as u8);
        ctx.set(x + 3, y + 2 + i as i32, white, black, rltk::to_cp437(')'));
        ctx.print(x + 5, y + 2 + i as i32, &name.name);

        equippable.push(entitiy);
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(VirtualKeyCode::Escape) => (ItemMenuResult::Cancel, None),
        Some(key) => {
            let selection = rltk::letter_to_option(key);
            if selection > -1 && selection < count {
                (ItemMenuResult::Selected, Some(equippable[selection as usize]))
            } else {
                (ItemMenuResult::NoResponse, None)
            }
        }
    }
}

pub fn draw_drop_item_menu(state: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    draw_inventory_menu(state, ctx, "Drop Which Item?")
}

pub fn draw_show_inventory_item_menu(state: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    draw_inventory_menu(state, ctx, "Inventory")
}

pub fn ranged_targeting(state: &mut State, ctx: &mut Rltk, range: i32) -> (ItemMenuResult, Option<Point>) {
    let black = RGB::named(rltk::BLACK);
    let yellow = RGB::named(rltk::YELLOW);
    let blue = RGB::named(rltk::BLUE);
    let red = RGB::named(rltk::RED);
    let cyan = RGB::named(rltk::CYAN);

    let player_entity = state.ecs.fetch::<Entity>();
    let player_position = state.ecs.fetch::<Point>();
    let viewsheds = state.ecs.read_storage::<Viewshed>();

    ctx.print_color(5, 0, yellow, black, "Select Target:");

    let mut available_cells = Vec::new();

    let visible = viewsheds.get(*player_entity).unwrap();

    for idx in visible.visible_tiles.iter() {
        let distance = rltk::DistanceAlg::Pythagoras.distance2d(*player_position, *idx);
        if distance <= range as f32 {
            ctx.set_bg(idx.x, idx.y, blue);
            available_cells.push(idx);
        }
    }
    let (mouse_x, mouse_y) = ctx.mouse_pos();

    if available_cells
        .iter()
        .any(|Point { x, y }| x == &mouse_x && y == &mouse_y)
    {
        if ctx.left_click {
            ctx.set_bg(mouse_x, mouse_y, cyan);
            (ItemMenuResult::Selected, Some(Point { x: mouse_x, y: mouse_y }))
        } else {
            (ItemMenuResult::NoResponse, None)
        }
    } else {
        ctx.set_bg(mouse_x, mouse_y, red);
        if ctx.left_click {
            (ItemMenuResult::Cancel, None)
        } else {
            (ItemMenuResult::NoResponse, None)
        }
    }
}

pub fn main_menu(selection: MainMenuSelection, ctx: &mut Rltk) -> MainMenuResult {
    let magenta = RGB::named(rltk::MAGENTA);
    let white = RGB::named(rltk::WHITE);
    let black = RGB::named(rltk::BLACK);

    ctx.print_color_centered(15, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "ROUGE");

    match selection {
        MainMenuSelection::NewGame => {
            ctx.print_color_centered(24, magenta, black, "Begin New Game");
            ctx.print_color_centered(25, white, black, "Load Game");
            ctx.print_color_centered(26, white, black, "Save and Quit");
        }
        MainMenuSelection::LoadGame => {
            ctx.print_color_centered(24, white, black, "Begin New Game");
            ctx.print_color_centered(25, magenta, black, "Load Game");
            ctx.print_color_centered(26, white, black, "Save and Quit");
        }

        MainMenuSelection::Quit => {
            ctx.print_color_centered(24, white, black, "Begin New Game");
            ctx.print_color_centered(25, white, black, "Load Game");
            ctx.print_color_centered(26, magenta, black, "Save and Quit");
        }
    }

    match ctx.key {
        None => MainMenuResult::NoSelection { selected: selection },
        Some(key) => match key {
            VirtualKeyCode::Escape => MainMenuResult::NoSelection {
                selected: MainMenuSelection::Quit,
            },
            VirtualKeyCode::Up => MainMenuResult::NoSelection {
                selected: match selection {
                    MainMenuSelection::NewGame => MainMenuSelection::Quit,
                    MainMenuSelection::LoadGame => MainMenuSelection::NewGame,
                    MainMenuSelection::Quit => MainMenuSelection::LoadGame,
                },
            },
            VirtualKeyCode::Down => MainMenuResult::NoSelection {
                selected: match selection {
                    MainMenuSelection::NewGame => MainMenuSelection::LoadGame,
                    MainMenuSelection::LoadGame => MainMenuSelection::Quit,
                    MainMenuSelection::Quit => MainMenuSelection::NewGame,
                },
            },
            VirtualKeyCode::Return => MainMenuResult::Selected { selected: selection },
            _ => MainMenuResult::NoSelection { selected: selection },
        },
    }
}
