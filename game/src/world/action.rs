use crate::{
    world::{
        data::*,
        spatial::{Layer, Location},
    },
    ExternalEvent, Message, World,
};
use coord_2d::ICoord;
use entity_table::Entity;
use rand::Rng;

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
        self.components.to_remove.insert(character, ());
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
}
