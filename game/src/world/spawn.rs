use crate::{
    world::{data::*, World},
    Entity,
};
use coord_2d::ICoord;
use entity_table::entity_data;
use rand::Rng;
use rgb_int::Rgb24;
use visible_area_detection::{vision_distance, Diminish, Light};

pub fn make_player() -> EntityData {
    EntityData {
        player: Some(()),
        character: Some(()),
        tile: Some(Tile::Player),
        light: Some(Light {
            colour: Rgb24::new(150, 150, 150),
            vision_distance: vision_distance::Circle::new_squared(200),
            diminish: Diminish::default().with_height(100.),
        }),
        health: Some(Meter::new(20, 20)),
        inventory: Some(Inventory::new(16)),
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

    fn spawn_entity<L: Into<Location>>(&mut self, location: L, entity_data: EntityData) -> Entity {
        let entity = self.entity_allocator.alloc();
        let location @ Location { layer, coord } = location.into();
        if let Err(e) = self.spatial_table.update(entity, location) {
            panic!("{:?}: There is already a {:?} at {:?}", e, layer, coord);
        }
        self.components.insert_entity_data(entity, entity_data);
        entity
    }

    pub fn spawn_wall(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Feature),
            entity_data! {
                tile: Tile::Wall,
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

    pub fn spawn_floor(&mut self, coord: ICoord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Floor),
            entity_data! {
                tile: Tile::Floor,
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

    pub fn spawn_zombie<R: Rng>(&mut self, coord: ICoord, rng: &mut R) -> Entity {
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
                resurrects_in: Meter::new_full(10),
                bump_damage: 1..=3,
            },
        )
    }
}
