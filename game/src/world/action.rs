use crate::{
    ExternalEvent, Message, World,
    world::{
        data::*,
        spatial::{Layer, Location},
    },
};
use coord_2d::ICoord;
use entity_table::Entity;
use rand::{Rng, seq::IndexedRandom};

impl World {
    pub fn damage_character<R: Rng>(
        &mut self,
        character: Entity,
        hit_points_to_lose: u32,
        rng: &mut R,
        external_events: &mut Vec<ExternalEvent>,
        message_log: &mut Vec<Message>,
    ) {
        if self.components.to_remove.contains(character) {
            // prevent cascading damage on explosions
            return;
        }
        let hit_points = self
            .components
            .health
            .get_mut(character)
            .expect("character lacks hit_points");
        if hit_points_to_lose >= hit_points.current() {
            hit_points.set_current(0);
            self.character_die(character, rng, external_events, message_log);
        } else {
            hit_points.decrease(hit_points_to_lose);
            if hit_points_to_lose > 0 {
                if let Some(&npc_type) = self.components.npc_type.get(character) {
                    message_log.push(Message::NpcHit {
                        npc_type,
                        damage: hit_points_to_lose,
                    });
                }
            }
            if self.components.split_on_damage.contains(character) && hit_points.current() > 1 {
                if let Some(coord) = self.spatial_table.coord_of(character) {
                    let copy_hit_poinst = hit_points.current() / 2;
                    hit_points.decrease(copy_hit_poinst);
                    let hit_points = *hit_points;
                    let mut copy_data = self.components.clone_entity_data(character);
                    copy_data.health = Some(Meter::new(copy_hit_poinst, hit_points.max()));
                    if let Some(copy_coord) = self.nearest_characterless_coord(coord) {
                        let copy_entity = self.entity_allocator.alloc();
                        self.components.insert_entity_data(copy_entity, copy_data);
                        let _ = self.spatial_table.update(
                            copy_entity,
                            Location {
                                coord: copy_coord,
                                layer: Some(Layer::Character),
                            },
                        );
                    }
                }
            }
        }
    }

    pub fn damage_player<R: Rng>(
        &mut self,
        character: Entity,
        hit_points_to_lose: u32,
        rng: &mut R,
        external_events: &mut Vec<ExternalEvent>,
        message_log: &mut Vec<Message>,
    ) {
        let player_entity = self.components.player.entities().next().unwrap();
        if self.components.to_remove.contains(player_entity) {
            // prevent cascading damage on explosions
            return;
        }
        if let Some(&npc_type) = self.components.npc_type.get(character) {
            message_log.push(Message::PlayerHit {
                attacker_npc_type: npc_type,
                damage: hit_points_to_lose,
            });
        }
        let hit_points = self
            .components
            .health
            .get_mut(player_entity)
            .expect("character lacks hit_points");
        if hit_points_to_lose >= hit_points.current() {
            hit_points.set_current(0);
            self.character_die(player_entity, rng, external_events, message_log);
        } else {
            hit_points.decrease(hit_points_to_lose);
        }
    }

    fn character_die<R: Rng>(
        &mut self,
        character: Entity,
        rng: &mut R,
        external_events: &mut Vec<ExternalEvent>,
        message_log: &mut Vec<Message>,
    ) {
        if let Some(&npc_type) = self.components.npc_type.get(character) {
            message_log.push(Message::NpcDies(npc_type));
        }
        if self.components.zombie.contains(character) {
            self.components.tile.insert(character, Tile::ZombieCorpse);
            self.components.character.remove(character);
        } else {
            self.components.to_remove.insert(character, ());
        }
    }

    pub fn player_bump_combat<R: Rng>(
        &mut self,
        character: Entity,
        rng: &mut R,
        external_events: &mut Vec<ExternalEvent>,
        message_log: &mut Vec<Message>,
    ) {
        let mut damage = rng.random_range(1..=2);
        self.damage_character(character, damage, rng, external_events, message_log);
    }

    fn drop_item(&mut self, item_entity: Entity, coord: ICoord) {
        if let Some(coord) = self.nearest_itemless_coord(coord) {
            let _ = self.spatial_table.update(
                item_entity,
                Location {
                    coord,
                    layer: Some(Layer::Item),
                },
            );
        }
    }

    fn resurrect(&mut self, entity: Entity) {
        let current_coord = self.spatial_table.coord_of(entity).unwrap();
        if let Some(coord) = self.nearest_characterless_coord(current_coord) {
            let _ = self.spatial_table.update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Character),
                },
            );
        } else {
            return;
        }
        if let Some(resurrects_in) = self.components.resurrects_in.get_mut(entity) {
            resurrects_in.set_current(resurrects_in.max());
        }
        if let Some(health) = self.components.health.get_mut(entity) {
            health.set_current(health.max());
        }
        self.components.character.insert(entity, ());
        self.components.tile.insert(entity, Tile::Zombie);
    }

    pub fn handle_resurrection(&mut self) {
        let mut to_resurrect = Vec::new();
        for (entity, resurrects_in) in self.components.resurrects_in.iter_mut() {
            if let Some(health) = self.components.health.get(entity) {
                if health.current() == 0 {
                    if resurrects_in.current() == 0 {
                        to_resurrect.push(entity);
                    } else {
                        resurrects_in.decrease(1);
                    }
                }
            }
        }
        for entity in to_resurrect {
            self.resurrect(entity);
        }
    }

    pub fn despawn_night_stalkers(&mut self, message_log: &mut Vec<Message>) {
        for entity in self.components.night_stalker.entities() {
            self.components.to_remove.insert(entity, ());
            message_log.push(Message::NightStalkerDespawn);
        }
    }

    pub fn handle_night_stalkers<R: Rng>(
        &mut self,
        time: crate::TimeOfDay,
        rng: &mut R,
        message_log: &mut Vec<Message>,
    ) {
        if time.is_night() {
            if time.minute() % 30 == 0 {
                let player_entity = self.components.player.entities().next().unwrap();
                let player_coord = self.spatial_table.coord_of(player_entity).unwrap();
                let spawn_candidates = self
                    .spatial_table
                    .grid_size()
                    .icoord_iter_row_major()
                    .filter(|coord| {
                        let layers = self.spatial_table.layers_at(*coord).unwrap();
                        if layers.feature.is_some() || layers.character.is_some() {
                            return false;
                        }
                        let distance2 = coord.distance2(player_coord);
                        distance2 > 40 && distance2 < 50
                    })
                    .collect::<Vec<_>>();
                if let Some(&coord) = spawn_candidates.choose(rng) {
                    self.spawn_night_stalker(coord);
                    message_log.push(Message::NightStalkerSpawn);
                }
            }
        } else {
            self.despawn_night_stalkers(message_log);
        }
    }
}
