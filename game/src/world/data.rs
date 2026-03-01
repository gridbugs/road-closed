pub use crate::world::spatial::{Layer, Location};
use entity_table::{Entity, declare_entity_module};
use rgb_int::Rgba32;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;
use visible_area_detection::{Light, vision_distance};

declare_entity_module! {
    components {
        player: (),
        character: (),
        tile: Tile,
        solid: (),
        door_state: DoorState,
        opacity: u8,
        light: Light<vision_distance::Circle>,
        health: Meter,
        energy: Meter,
        food: Meter,
        car_fuel: Meter,
        resurrects_in: Meter,
        npc: Npc,
        npc_type: NpcType,
        item: Item,
        inventory: Inventory,
        to_remove: (),
        split_on_damage: (),
        colour_hint: Rgba32,
        bump_damage: RangeInclusive<u32>,
        difficult: (),
        your_car: (),
        grass: (),
        zombie: (),
        night_stalker: (),
        slow: u64,
    }
}
pub use components::{Components, EntityData, EntityUpdate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Tile {
    Player,
    DeadPlayer,
    Ground,
    Road,
    Tree,
    DeadTree,
    FallenTree,
    Wall,
    DoorClosed,
    DoorOpen,
    Zombie,
    ZombieCorpse,
    Debris,
    Item(Item),
    Car(char),
    Grass,
    CrushedGrass,
    CabinWall,
    Window,
    Floor,
    NightStalker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DoorState {
    Open,
    Closed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Meter {
    current: u32,
    max: u32,
}

impl Meter {
    pub fn new(current: u32, max: u32) -> Self {
        Self { current, max }
    }
    pub fn new_full(max: u32) -> Self {
        Self::new(max, max)
    }
    pub fn current_and_max(&self) -> (u32, u32) {
        (self.current, self.max)
    }
    pub fn current(&self) -> u32 {
        self.current
    }
    pub fn max(&self) -> u32 {
        self.max
    }
    pub fn set_current(&mut self, to: u32) {
        self.current = to.min(self.max);
    }
    pub fn decrease(&mut self, by: u32) {
        self.current = self.current.saturating_sub(by);
    }
    pub fn increase(&mut self, by: u32) {
        self.set_current(self.current + by);
    }
    pub fn set_max(&mut self, to: u32) {
        self.max = to;
        self.set_current(self.current);
    }
    pub fn is_empty(&self) -> bool {
        self.current == 0
    }
    pub fn is_full(&self) -> bool {
        self.current == self.max
    }
    pub fn fill(&mut self) {
        self.current = self.max;
    }
    pub fn clear(&mut self) {
        self.current = 0;
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NpcType {
    Zombie,
    NightStalker,
}

impl NpcType {
    pub fn tile(self) -> Tile {
        match self {
            Self::Zombie => Tile::Zombie,
            Self::NightStalker => Tile::NightStalker,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Item {
    MedKit,
    Firewood,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Inventory {
    pub items: Vec<Option<EntityData>>,
}

impl Inventory {
    pub fn new(size: usize) -> Self {
        Self {
            items: (0..size).map(|_| None).collect(),
        }
    }

    pub fn has_free_slot(&self) -> bool {
        for entry in self.items.iter() {
            if entry.is_none() {
                return true;
            }
        }
        false
    }

    pub fn first_free_slot(&mut self) -> Option<&mut Option<EntityData>> {
        for entry in self.items.iter_mut() {
            if entry.is_none() {
                return Some(entry);
            }
        }
        None
    }

    pub fn size(&self) -> usize {
        self.items.len()
    }

    pub fn get(&self, i: usize) -> Option<&EntityData> {
        self.items[i].as_ref()
    }

    pub fn remove(&mut self, i: usize) -> Option<EntityData> {
        self.items[i].take()
    }

    pub fn items(&self) -> &[Option<EntityData>] {
        &self.items
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Disposition {
    Hostile,
    Afraid,
    Thief,
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NpcMovement {
    pub can_traverse_difficult: bool,
    pub can_open_doors: bool,
}

impl NpcMovement {
    pub const ALL: &'static [Self] = &[
        NpcMovement {
            can_traverse_difficult: false,
            can_open_doors: false,
        },
        NpcMovement {
            can_traverse_difficult: false,
            can_open_doors: true,
        },
        NpcMovement {
            can_traverse_difficult: true,
            can_open_doors: false,
        },
        NpcMovement {
            can_traverse_difficult: true,
            can_open_doors: true,
        },
    ];
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Npc {
    pub disposition: Disposition,
    pub movement: NpcMovement,
}
