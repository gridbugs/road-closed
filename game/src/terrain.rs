use crate::world::{World, data::*};
use coord_2d::{ICoord, UCoord};
use direction::Direction;
use procgen::city::{Map, TentacleSpec, Tile};
use rand::{Rng, prelude::IndexedRandom, seq::SliceRandom};

pub struct Terrain {
    pub world: World,
}

impl Terrain {
    #[allow(unused)]
    pub fn generate_text() -> Self {
        let txt = include_str!("terrain.txt");
        let rows = txt.split('\n').collect::<Vec<_>>();
        let mut world = World::new(UCoord::new(rows[0].len() as u32, rows.len() as u32));
        let mut rng = rand::rng();
        for (y, row) in rows.into_iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let coord = ICoord::new(x as i32, y as i32);
                match ch {
                    '.' => {
                        world.spawn_ground(coord);
                    }
                    ',' => {
                        world.spawn_road(coord);
                    }
                    '&' => {
                        world.spawn_tree(coord);
                    }
                    '|' | '/' | '\\' | '=' | '_' | '[' | ']' | ' ' => {
                        world.spawn_car_part(coord, ch);
                    }
                    _ => log::warn!("unexpected char: {}", ch),
                }
            }
        }
        Self { world }
    }
}
