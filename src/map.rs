use crate::rect::Rect;
use crate::spawner::{random_item, random_monster};
use crate::{MAP_HEIGHT, MAP_WIDTH};
use rltk::{Algorithm2D, BaseMap, Console, Point, RandomNumberGenerator, Rltk, RGB};
use specs::{Entity, World, WorldExt};
use std::cmp::{max, min};

const MAX_ROOMS: i32 = 30;
const MIN_SIZE: i32 = 6;
const MAX_SIZE: i32 = 10;
const MAX_MONSTERS_PER_ROOM: i32 = 4;
const MAX_ITEMS_PER_ROOM: i32 = 1;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

#[derive(Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub tile_content: Vec<Vec<Entity>>,
    pub dimensions: usize,
}

impl Map {
    pub fn draw(&self, ctx: &mut Rltk) {
        let visible = RGB::from_u8(192, 192, 192);
        let memory = RGB::from_u8(105, 105, 105);
        let bg = RGB::from_f32(0.0, 0.0, 0.0);
        let wall = rltk::to_cp437('#');
        let floor = rltk::to_cp437('.');

        for (idx, tile) in self.tiles.iter().enumerate() {
            if self.revealed_tiles[idx] {
                let fg = if self.visible_tiles[idx] { visible } else { memory };
                let glyph = match tile {
                    TileType::Floor => floor,
                    TileType::Wall => wall,
                };
                let (x, y) = self.idx_to_xy(idx);
                ctx.set(x, y, fg, bg, glyph);
            }
        }
    }

    pub fn xy_to_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    pub fn point_to_idx(&self, point: Point) -> usize {
        (point.y as usize * self.width as usize) + point.x as usize
    }

    pub fn idx_to_xy(&self, idx: usize) -> (i32, i32) {
        (idx as i32 % self.width, idx as i32 / self.width)
    }

    pub fn new() -> Map {
        let dimensions = (MAP_HEIGHT * MAP_WIDTH) as usize;
        Map {
            dimensions,
            tiles: vec![TileType::Wall; dimensions],
            rooms: Vec::new(),
            width: MAP_WIDTH,
            height: MAP_HEIGHT,
            revealed_tiles: vec![false; dimensions],
            visible_tiles: vec![false; dimensions],
            blocked: vec![false; dimensions],
            tile_content: vec![Vec::new(); dimensions],
        }
    }

    pub fn new_map_with_rooms_and_corridors() -> Map {
        let mut map = Self::new();

        let mut rng = RandomNumberGenerator::new();
        for _i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false
                }
            }
            if ok {
                map.apply_room_to_map(&new_room);
                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                    map.apply_tunnels(prev_x, prev_y, new_x, new_y);
                }
                map.rooms.push(new_room);
            }
        }
        map
    }

    pub fn populate_room(ecs: &mut World, room: &Rect) {
        let mut monster_spawn_points = Vec::new();
        let mut item_spawn_points = Vec::new();

        {
            let mut rng = ecs.write_resource::<RandomNumberGenerator>();

            // TODO: REFACTOR THOSE, MAKE IT A GENERIC FACTORY
            for _i in 0..rng.roll_dice(1, MAX_MONSTERS_PER_ROOM) {
                let mut added = false;
                while !added {
                    let x = room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1));
                    let y = room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1));

                    let idx = (y * MAP_WIDTH) + x;
                    if !monster_spawn_points.contains(&idx) {
                        monster_spawn_points.push(idx);
                        added = true;
                    }
                }
            }

            for _i in 0..rng.roll_dice(1, MAX_ITEMS_PER_ROOM) {
                let mut added = false;
                while !added {
                    let x = room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1));
                    let y = room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1));

                    let idx = (y * MAP_WIDTH) + x;
                    if !item_spawn_points.contains(&idx) {
                        item_spawn_points.push(idx);
                        added = true;
                    }
                }
            }
        }

        for idx in monster_spawn_points.iter() {
            let x = idx % MAP_WIDTH;
            let y = idx / MAP_WIDTH;
            random_monster(ecs, x, y);
        }
        for idx in item_spawn_points.iter() {
            let x = idx % MAP_WIDTH;
            let y = idx / MAP_WIDTH;
            random_item(ecs, x, y);
        }
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }
        let idx = self.xy_to_idx(x, y);
        !self.blocked[idx]
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = self.xy_to_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_tunnels(&mut self, prev_x: i32, prev_y: i32, new_x: i32, new_y: i32) {
        for x in min(prev_x, new_x)..=max(prev_x, new_x) {
            for y in min(prev_y, new_y)..=max(prev_y, new_y) {
                let idx = self.xy_to_idx(x, new_y);
                if idx > 0 && idx < (self.width * self.height) as usize {
                    self.tiles[idx as usize] = TileType::Floor
                }
                let idx = self.xy_to_idx(new_x, y);
                if idx > 0 && idx < (self.width * self.height) as usize {
                    self.tiles[idx as usize] = TileType::Floor
                }
            }
        }
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point {
            x: self.width,
            y: self.height,
        }
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> Vec<(usize, f32)> {
        let mut exits = Vec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0))
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0))
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0))
        };

        if self.is_exit_valid(x - 1, y - 1) {
            exits.push((idx - w - 1, 1.45))
        };
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push((idx - w + 1, 1.45))
        };
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push((idx + w - 1, 1.45))
        };
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push((idx + w + 1, 1.45))
        };

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let width = self.width as usize;

        let p1 = Point::new(idx1 % width, idx1 / width);
        let p2 = Point::new(idx2 % width, idx2 / width);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}
