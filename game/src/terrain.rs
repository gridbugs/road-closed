use crate::world::World;
use coord_2d::{ICoord, UCoord};
use rand::{Rng, seq::SliceRandom};

pub struct Terrain {
    pub world: World,
    pub player_spawn: ICoord,
    empty_space_far_from_player: Vec<ICoord>,
    cabin_centers: Vec<ICoord>,
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
            empty_space_far_from_player: Vec::new(),
            cabin_centers: Vec::new(),
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
        //world.spawn_zombie(player_spawn + ICoord::new(0, 2));
        //world.spawn_item(player_spawn + ICoord::new(0, 2), Item::MedKit);
        //world.spawn_item(player_spawn + ICoord::new(0, 3), Item::Firewood);
        //world.spawn_item(player_spawn + ICoord::new(0, 4), Item::Fruit);
        //world.spawn_item(player_spawn + ICoord::new(0, 5), Item::Coffee);
        //world.spawn_item(player_spawn + ICoord::new(0, 6), Item::FuelCan);
        //world.spawn_item(player_spawn + ICoord::new(0, 7), Item::Food);
        Self {
            player_spawn,
            world,
            empty_space_far_from_player: map1.empty_space_far_from_player,
            cabin_centers: map1.cabin_centres,
        }
    }

    pub fn generate_end<R: Rng>(rng: &mut R) -> Self {
        use procgen::end::{Map1, Tile as MapTile};
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
                Typewriter => {
                    world.spawn_typewriter(coord);
                }
            }
        }
        let player_spawn = player_coord.unwrap();
        Self {
            player_spawn,
            world,
            empty_space_far_from_player: Vec::new(),
            cabin_centers: Vec::new(),
        }
    }

    pub fn generate_swamp<R: Rng>(rng: &mut R) -> Self {
        use procgen::forest::{Map1, Tile as MapTile};
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
                BridgeRailing => {
                    world.spawn_bridge_railing(coord);
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
                Water => {
                    world.spawn_water(coord);
                }
                WaterTree => {
                    world.spawn_water(coord);
                    world.spawn_tree(coord);
                }
            }
        }
        let player_spawn = player_coord.unwrap();
        Self {
            player_spawn,
            world,
            empty_space_far_from_player: map1.empty_space_far_from_player,
            cabin_centers: map1.cabin_centres,
        }
    }

    pub fn generate_mountain_pass<R: Rng>(rng: &mut R) -> Self {
        use procgen::mountain_pass::{Map1, Tile as MapTile};
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
                CliffFace => {
                    world.spawn_cliff_face(coord);
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
        Self {
            player_spawn,
            world,
            empty_space_far_from_player: map1.empty_space_far_from_player,
            cabin_centers: map1.cabin_centers,
        }
    }

    pub fn populate<R: Rng>(&mut self, distance_remaining: u32, rng: &mut R) {
        use crate::world::data::Armour::*;
        use crate::world::data::Item::*;
        use crate::world::data::NpcType::*;
        use crate::world::data::Weapon::*;
        let mut cabin_center_items = match distance_remaining {
            0..500 => vec![
                Weapon(Axe),
                Weapon(Axe),
                Weapon(Knife),
                Armour(HeavyArmour),
                Armour(LightArmour),
                FuelCan,
                FuelCan,
                Coffee,
                MedKit,
                Food,
            ],
            500..1000 => vec![
                Weapon(Axe),
                Weapon(Knife),
                Armour(LightArmour),
                FuelCan,
                FuelCan,
                Coffee,
                MedKit,
                Food,
                FuelCan,
                FuelCan,
                Coffee,
                MedKit,
                Food,
            ],
            1000..1500 => vec![
                Weapon(Knife),
                Armour(LightArmour),
                Armour(Overalls),
                FuelCan,
                FuelCan,
                FuelCan,
                FuelCan,
                Coffee,
                MedKit,
                Food,
                FuelCan,
                FuelCan,
                Coffee,
                MedKit,
                Food,
            ],
            1500.. => vec![
                Weapon(Knife),
                Armour(Overalls),
                FuelCan,
                FuelCan,
                FuelCan,
                FuelCan,
                FuelCan,
                FuelCan,
                Coffee,
                MedKit,
                Food,
            ],
        };
        cabin_center_items.shuffle(rng);
        for &coord in &self.cabin_centers {
            if let Some(item) = cabin_center_items.pop() {
                self.world.spawn_item(coord, item);
            }
        }
        self.empty_space_far_from_player.shuffle(rng);
        let num_fruit = 20;
        for _ in 0..num_fruit {
            if let Some(coord) = self.empty_space_far_from_player.pop() {
                self.world.spawn_item(coord, Fruit);
            }
        }
        let mut npcs = match distance_remaining {
            0..500 => vec![
                Zombie, Zombie, Zombie, Climber, Climber, Climber, Slime, Slime, Slime, Drainer,
                Drainer, Drainer,
            ],
            500..1000 => vec![
                Zombie, Zombie, Zombie, Zombie, Zombie, Climber, Climber, Climber, Slime, Slime,
                Drainer,
            ],
            1000..1500 => vec![
                Zombie, Zombie, Zombie, Zombie, Zombie, Zombie, Zombie, Climber, Climber, Slime,
            ],
            1500.. => vec![
                Zombie, Zombie, Zombie, Zombie, Zombie, Zombie, Zombie, Zombie, Slime,
            ],
        };
        npcs.shuffle(rng);
        for npc in npcs {
            if let Some(coord) = self.empty_space_far_from_player.pop() {
                match npc {
                    Zombie => {
                        self.world.spawn_zombie(coord);
                    }
                    Slime => {
                        self.world.spawn_slime(coord);
                    }
                    Drainer => {
                        self.world.spawn_drainer(coord);
                    }
                    Climber => {
                        self.world.spawn_climber(coord);
                    }
                    _ => (),
                }
            }
        }
    }
}
