use crate::world::{data::*, World};
use coord_2d::{ICoord, UCoord};
use direction::Direction;
use procgen::city::{Map, TentacleSpec, Tile};
use rand::{prelude::IndexedRandom, seq::SliceRandom, Rng};

pub struct Terrain {
    pub world: World,
}

impl Terrain {
    #[allow(unused)]
    pub fn generate_text() -> Self {
        let txt = include_str!("terrain.txt");
        let rows = txt.split('\n').collect::<Vec<_>>();
        let mut world = World::new(UCoord::new(50, 25));
        let mut rng = rand::rng();
        for (y, row) in rows.into_iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let coord = ICoord::new(x as i32, y as i32);
                let floor_entity = world.spawn_floor(coord);
                match ch {
                    '.' => (),
                    _ => log::warn!("unexpected char: {}", ch),
                }
            }
        }
        Self { world }
    }
}
