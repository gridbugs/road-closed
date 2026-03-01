use crate::{
    World,
    world::{data::*, spatial::Layers},
};
use coord_2d::ICoord;
use direction::CardinalDirection;
use entity_table::Entity;
use rand::{Rng, prelude::IndexedRandom, seq::SliceRandom};

impl World {
    pub fn entity_coord(&self, entity: Entity) -> Option<ICoord> {
        self.spatial_table.coord_of(entity)
    }

    pub fn can_npc_traverse_feature_at_coord_with_movement(
        &self,
        coord: ICoord,
        movement: NpcMovement,
    ) -> bool {
        if let Some(layers) = self.spatial_table.layers_at(coord) {
            if let Some(feature) = layers.feature {
                !self.components.solid.contains(feature)
                    || movement.can_open_doors && self.components.door_state.contains(feature)
                    || movement.can_traverse_difficult
                        && self.components.difficult.contains(feature)
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn can_npc_traverse_feature_at_coord_with_entity(
        &self,
        coord: ICoord,
        npc_entity: Entity,
    ) -> bool {
        let npc = self.components.npc.get(npc_entity).expect("not an npc");
        self.can_npc_traverse_feature_at_coord_with_movement(coord, npc.movement)
    }

    pub fn is_npc_at_coord(&self, coord: ICoord) -> bool {
        if let Some(layers) = self.spatial_table.layers_at(coord) {
            if let Some(character) = layers.character {
                self.components.npc.contains(character)
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn get_opacity(&self, coord: ICoord) -> u8 {
        if let Some(&Layers {
            feature: Some(feature_entity),
            ..
        }) = self.spatial_table.layers_at(coord)
        {
            self.components
                .opacity
                .get(feature_entity)
                .cloned()
                .unwrap_or(0)
        } else {
            0
        }
    }
    pub fn character_at_coord(&self, coord: ICoord) -> Option<Entity> {
        if let Some(layers) = self.spatial_table.layers_at(coord) {
            layers.character
        } else {
            None
        }
    }
    pub fn entity_npc(&self, entity: Entity) -> Option<&Npc> {
        self.components.npc.get(entity)
    }

    pub fn nearest_itemless_coord(&self, start: ICoord) -> Option<ICoord> {
        use std::collections::{HashSet, VecDeque};
        if let Some(layers) = self.spatial_table.layers_at(start) {
            if layers.feature.is_none() && layers.item.is_none() {
                return Some(start);
            }
        }
        let mut seen = HashSet::new();
        seen.insert(start);
        let mut queue = VecDeque::new();
        queue.push_back(start);
        while let Some(coord) = queue.pop_front() {
            for d in CardinalDirection::all() {
                let coord = coord + d.coord();
                if seen.insert(coord) {
                    if let Some(layers) = self.spatial_table.layers_at(coord) {
                        if layers.feature.is_none() {
                            if layers.item.is_none() {
                                return Some(coord);
                            } else {
                                queue.push_back(coord);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn nearest_characterless_coord(&self, start: ICoord) -> Option<ICoord> {
        use std::collections::{HashSet, VecDeque};
        if self
            .spatial_table
            .layers_at_checked(start)
            .character
            .is_none()
        {
            return Some(start);
        }
        let mut seen = HashSet::new();
        seen.insert(start);
        let mut queue = VecDeque::new();
        queue.push_back(start);
        while let Some(coord) = queue.pop_front() {
            for d in CardinalDirection::all() {
                let coord = coord + d.coord();
                if seen.insert(coord) {
                    if let Some(layers) = self.spatial_table.layers_at(coord) {
                        if layers.feature.is_none() {
                            if layers.character.is_none() {
                                return Some(coord);
                            } else {
                                queue.push_back(coord);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn player_inventory_item_index(&self, item: Item) -> Option<usize> {
        let player = self.components.player.entities().next().unwrap();
        let inventory = self.components.inventory.get(player).unwrap();
        for (i, entity_data_opt) in inventory.items().iter().enumerate() {
            if let Some(entity_data) = entity_data_opt.as_ref() {
                if let Some(current_item) = entity_data.item {
                    if current_item == item {
                        return Some(i);
                    }
                }
            }
        }
        None
    }

    pub fn is_game_over(&self) -> bool {
        if let Some(player_entity) = self.components.player.entities().next() {
            if self.components.to_remove.contains(player_entity) {
                return true;
            }
            if let Some(health) = self.components.health.get(player_entity) {
                if health.current() == 0 {
                    return true;
                }
            }
            false
        } else {
            true
        }
    }

    pub fn line_distance_stopping_at_solid(&self, from: ICoord, to: ICoord) -> Option<usize> {
        let mut count = 0;
        for coord in line_2d::coords_between(from, to) {
            if let Some(Layers {
                feature: Some(feature),
                ..
            }) = self.spatial_table.layers_at(coord)
            {
                if self.components.solid.contains(*feature)
                    && !self.components.difficult.contains(*feature)
                {
                    return None;
                }
            }
            count += 1;
        }
        Some(count)
    }

    pub fn random_characterless_coord<R: Rng>(&self, rng: &mut R) -> Option<ICoord> {
        let candidates = self
            .spatial_table
            .grid_size()
            .icoord_iter_row_major()
            .filter(|coord| {
                let layers = self.spatial_table.layers_at_checked(*coord);
                layers.character.is_none() && layers.feature.is_none()
            })
            .collect::<Vec<_>>();
        candidates.choose(rng).copied()
    }
}
