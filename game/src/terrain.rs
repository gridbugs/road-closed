use crate::world::{World, data::*};
use coord_2d::{ICoord, UCoord};
use direction::Direction;
use procgen::city::{Map, TentacleSpec, Tile};
use rand::{Rng, prelude::IndexedRandom, seq::SliceRandom};

pub struct Terrain {
    pub world: World,
    pub player_spawn: ICoord,
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
        Self {
            player_spawn: ICoord::new(0, 0),
            world,
        }
    }

    pub fn generate_pine_plantation<R: Rng>(rng: &mut R) -> Self {
        use procgen::pine_plantation::{Map1, Tile as MapTile};
        let map1 = Map1::generate(rng);
        let mut player_coord = None;
        let mut world = World::new(map1.grid.size());
        for (coord, &cell) in map1.grid.enumerate() {
            use MapTile::*;
            match cell {
                Player => {
                    player_coord = Some(coord);
                    world.spawn_road(coord);
                }
                Car(ch) => {
                    world.spawn_car_part(coord, ch);
                }
                Ground => {
                    world.spawn_ground(coord);
                }
                Road => {
                    world.spawn_road(coord);
                }
                Tree => {
                    world.spawn_tree(coord);
                }
                DeadTree => {
                    world.spawn_dead_tree(coord);
                }
                FallenTree => {
                    world.spawn_fallen_tree(coord);
                }
                FireWood => {
                    world.spawn_firewood(coord);
                }
                Wall => {
                    world.spawn_cabin_wall(coord);
                }
                Floor => {
                    world.spawn_floor(coord);
                }
                Door => {
                    world.spawn_door(coord);
                }
                Window => {
                    world.spawn_window(coord);
                }
                Grass => {
                    world.spawn_grass(coord);
                }
            }
        }
        let player_spawn = player_coord.unwrap();
        world.spawn_zombie(player_spawn + ICoord::new(0, 5));
        Self {
            player_spawn,
            world,
        }
    }
}
