use crate::{
    Entity,
    world::{World, data::*},
};
use coord_2d::ICoord;
use entity_table::entity_data;
use rgb_int::Rgb24;
use visible_area_detection::{Diminish, Light, vision_distance};

pub fn make_player() -> EntityData {
    EntityData {
        player: Some(()),
        character: Some(()),
        tile: Some(Tile::Player),
        light: Some(Light {
            colour: Rgb24::new(150, 150, 150),
            vision_distance: vision_distance::Circle::new_squared(300),
            diminish: Diminish::default().with_height(100.),
        }),
        health: Some(Meter::new(20, 20)),
        energy: Some(Meter::new(20, 20)),
        food: Some(Meter::new(20, 20)),
        car_fuel: Some(Meter::new(20, 20)),
        inventory: Some(Inventory::new(6)),
        car_inventory: Some(Inventory::new(12)),
        ..Default::default()
    }
}

impl World {
    pub fn insert_entity_data(&mut self, location: Location, entity_data: EntityData) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table.update(entity, location).unwrap();
        self.components.insert_entity_data(entity, entity_data);
        entity
    }

    pub fn spawn_entity<L: Into<Location>>(
        &mut self,
        location: L,
        entity_data: EntityData,
    ) -> Entity {
        let entity = self.entity_allocator.alloc();
        let location @ Location { layer, coord } = location.into();
        if let Err(e) = self.spatial_table.update(entity, location) {
            panic!("{:?}: There is already a {:?} at {:?}", e, layer, coord);
        }
        self.components.insert_entity_data(entity, entity_data);
        entity
    }

    pub fn spawn_cabin_wall(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Feature),
            entity_data! {
                tile: Tile::CabinWall,
                solid: (),
                opacity: 255,
            },
        )
    }

    pub fn spawn_bridge_railing(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Feature),
            entity_data! {
                tile: Tile::BridgeRailing,
                solid: (),
            },
        )
    }

    pub fn spawn_cliff_face(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Feature),
            entity_data! {
                tile: Tile::CliffFace,
                solid: (),
                opacity: 255,
            },
        )
    }

    pub fn spawn_debris(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Feature),
            entity_data! {
                tile: Tile::Debris,
                solid: (),
                difficult: (),
            },
        )
    }

    pub fn spawn_ground(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Floor),
            entity_data! {
                tile: Tile::Ground,
            },
        )
    }

    pub fn spawn_floor(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Floor),
            entity_data! {
                tile: Tile::Floor,
            },
        )
    }

    pub fn spawn_water(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Floor),
            entity_data! {
                tile: Tile::Water,
            },
        )
    }

    pub fn spawn_grass(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Floor),
            entity_data! {
                tile: Tile::Grass,
                grass: (),
            },
        )
    }

    pub fn spawn_road(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Floor),
            entity_data! {
                tile: Tile::Road,
            },
        )
    }

    pub fn spawn_tree(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Feature),
            entity_data! {
                tile: Tile::Tree,
                opacity: 127,
                solid: (),
            },
        )
    }

    pub fn spawn_dead_tree(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Feature),
            entity_data! {
                tile: Tile::DeadTree,
                opacity: 63,
                solid: (),
            },
        )
    }

    pub fn spawn_fallen_tree(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Feature),
            entity_data! {
                tile: Tile::FallenTree,
                difficult: (),
                solid: (),
            },
        )
    }

    pub fn spawn_firewood(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Item),
            entity_data! {
                tile: Tile::Item(Item::Firewood),
                item: Item::Firewood,
            },
        )
    }

    pub fn spawn_window(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Feature),
            entity_data! {
                tile: Tile::Window,
                difficult: (),
                solid: (),
            },
        )
    }

    pub fn spawn_door(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Feature),
            entity_data! {
                tile: Tile::DoorClosed,
                solid: (),
                door_state: DoorState::Closed,
                opacity: 255,
            },
        )
    }

    pub fn spawn_item(&mut self, coord: ICoord, item: Item) -> Entity {
        let mut data = entity_data! {
            tile: Tile::Item(item),
            item,
        };
        self.spawn_entity((coord, Layer::Item), data)
    }

    pub fn spawn_item_no_coord(&mut self, item: Item) -> Entity {
        let mut data = entity_data! {
            tile: Tile::Item(item),
            item,
        };
        let entity = self.entity_allocator.alloc();
        self.components.insert_entity_data(entity, data);
        entity
    }

    pub fn spawn_zombie(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Character),
            entity_data! {
                tile: Tile::Zombie,
                npc: Npc {
                    disposition: Disposition::Hostile,
                    movement: NpcMovement {
                        can_traverse_difficult: false,
                        can_open_doors: false,
                    },
                },
                character: (),
                npc_type: NpcType::Zombie,
                health: Meter::new_full(4),
                resurrects_in: Meter::new_full(5),
                bump_damage: 1..=4,
                zombie: (),
                slow: 2,
            },
        )
    }

    pub fn spawn_slime(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Character),
            entity_data! {
                tile: Tile::Slime,
                npc: Npc {
                    disposition: Disposition::Hostile,
                    movement: NpcMovement {
                        can_traverse_difficult: false,
                        can_open_doors: false,
                    },
                },
                character: (),
                npc_type: NpcType::Slime,
                health: Meter::new_full(10),
                bump_damage: 2..=3,
                split_on_damage: (),
            },
        )
    }

    pub fn spawn_night_stalker(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Character),
            entity_data! {
                tile: Tile::NightStalker,
                npc: Npc {
                    disposition: Disposition::Hostile,
                    movement: NpcMovement {
                        can_traverse_difficult: true,
                        can_open_doors: true,
                    },
                },
                character: (),
                npc_type: NpcType::NightStalker,
                health: Meter::new_full(10),
                bump_damage: 4..=6,
                night_stalker: (),
            },
        )
    }

    pub fn spawn_car_part(&mut self, coord: ICoord, ch: char) -> Entity {
        self.spawn_entity(
            (coord, Layer::Feature),
            entity_data! {
                tile: Tile::Car(ch),
                your_car: (),
                difficult: (),
                solid: (),
            },
        )
    }

    pub fn spawn_typewriter(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Feature),
            entity_data! {
                tile: Tile::Typewriter,
                typewriter: (),
            },
        )
    }
}
