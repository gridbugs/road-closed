use direction::CardinalDirections;
pub use direction::{CardinalDirection, Direction};
pub use entity_table::{ComponentTable, Entity, entity_data, entity_update};
pub use grid_2d::{Grid, ICoord, UCoord};
pub use grid_search_cardinal::distance_map;
pub use line_2d::{self, coords_between, coords_between_cardinal};
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
pub use rgb_int::{Rgb24, Rgba32};
use serde::{Deserialize, Serialize};
pub use spatial_table::UpdateError;
use std::time::Duration;

pub use visible_area_detection::{
    CellVisibility, Light, VisibilityGrid, World as VisibleWorld, vision_distance::Circle,
};

mod terrain;
mod world;
use terrain::Terrain;
mod ai;
pub mod witness;

use ai::{Agent, AiContext};
use world::{
    World,
    data::{DoorState, EntityData, EntityUpdate},
    spatial::Layers,
};
pub use world::{
    data::{Item, Layer, Location, Meter, NpcType, Tile},
    spatial::LayerTable,
};

use crate::world::data::Inventory;

#[derive(Debug, Clone, Copy)]
pub struct Omniscient;

#[derive(Debug, Clone, Copy, Default)]
pub struct Config {
    pub omniscient: Option<Omniscient>,
    pub demo: bool,
    pub debug: bool,
}
impl Config {
    pub const OMNISCIENT: Option<Omniscient> = Some(Omniscient);
}

pub const MAX_ORGANS: usize = 8;

/// Events which the game can report back to the io layer so it can
/// respond with a sound/visual effect.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum ExternalEvent {
    Death,
    Leave,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Mode {
    Driving,
    Walking,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum PeriodOfDay {
    Dawn,
    Morning,
    Afternoon,
    Dusk,
    Night,
}

impl std::fmt::Display for PeriodOfDay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PeriodOfDay::Dawn => "dawn",
            PeriodOfDay::Morning => "morning",
            PeriodOfDay::Afternoon => "afternoon",
            PeriodOfDay::Dusk => "dusk",
            PeriodOfDay::Night => "night",
        };
        write!(f, "{}", s)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct TimeOfDay {
    hour: u8,
    minute: u8,
}

impl TimeOfDay {
    #[must_use]
    pub fn add_hours(&self, hours: u32) -> Self {
        Self {
            hour: ((self.hour as u32 + hours) % 24) as u8,
            minute: self.minute,
        }
    }

    #[must_use]
    pub fn add_minutes(&self, minutes: u32) -> Self {
        let minutes = self.minute as u32 + minutes;
        let hours = minutes / 60;
        let minute = (minutes % 60) as u8;
        Self {
            minute,
            ..self.add_hours(hours)
        }
    }
    pub fn period(&self) -> PeriodOfDay {
        use PeriodOfDay::*;
        match self.hour {
            0..=4 => Night,
            5 => Dawn,
            6..=11 => Morning,
            12..=19 => Afternoon,
            20 => Dusk,
            21.. => Night,
        }
    }
    pub fn is_night(&self) -> bool {
        match self.period() {
            PeriodOfDay::Night => true,
            _ => false,
        }
    }
    pub fn minute(&self) -> u8 {
        self.minute
    }
}

impl std::fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}:{:02} ({})", self.hour, self.minute, self.period())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Countdown {
    resets_to: u32,
    value: u32,
}

impl Countdown {
    fn new(resets_to: u32) -> Self {
        Self {
            resets_to,
            value: resets_to,
        }
    }

    fn tick(&mut self) -> bool {
        self.value -= 1;
        let ret = self.value == 0;
        if ret {
            self.value = self.resets_to;
        }
        ret
    }

    fn reset(&mut self) {
        self.value = self.resets_to;
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Wait,
    PlayerHit {
        attacker_npc_type: NpcType,
        damage: u32,
    },
    NpcHit {
        npc_type: NpcType,
        damage: u32,
    },
    KickZombieCorpse,
    DestroyZombieCorpse,
    YouDie,
    NpcDies(NpcType),
    OpenDoor,
    CloseDoor,
    ActionError(ActionError),
    GetItem(Item),
    DropItem(Item),
    OutOfFuel,
    TooTiredToDrive,
    GetOutOfCar,
    GetInCar,
    NightStalkerSpawn,
    NightStalkerDespawn,
    CanOnlySleepAtNight,
    TransferItemToCar(Item),
    TransferItemFromCar(Item),
    AboutToPassOut,
    PassOut,
    DamageFromHunger,
    ApplyItem(Item),
    MustBeNextToCarToRefuel,
    MakeWish,
}

#[derive(Debug, Clone, Copy)]
pub enum MenuChoice {
    Empty,
    DropItem(usize),
    ApplyItem(usize),
    TakeItemFromCar {
        car_inventory_slot_index: usize,
    },
    TransferItemToCar {
        car_inventory_slot_index: usize,
        player_inventor_slot_index: usize,
    },
}

#[derive(Debug, Clone)]
pub struct Menu {
    pub choices: Vec<MenuChoice>,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Victory {}

#[derive(Debug, Clone, Copy)]
pub enum GameOverReason {
    YouDied,
}

#[derive(Debug)]
pub enum GameControlFlow {
    GameOver(GameOverReason),
    Win,
    Menu(Menu),
}

pub struct PlayerStats {
    pub health: Meter,
    pub energy: Meter,
    pub car_fuel: Meter,
    pub food: Meter,
}

#[derive(Clone, Copy, Debug)]
pub enum Input {
    Walk(CardinalDirection),
    Wait,
    Get,
    ContinueDriving,
    StopDriving,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum TerrainType {
    Start,
    PinePlantation,
    End,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct VisibleEntity {
    pub tile: Option<Tile>,
    pub colour_hint: Option<Rgba32>,
    pub health: Option<Meter>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VisibleCellData {
    pub tiles: LayerTable<VisibleEntity>,
}
impl Default for VisibleCellData {
    fn default() -> Self {
        Self {
            tiles: LayerTable {
                floor: Default::default(),
                feature: Default::default(),
                character: Default::default(),
                item: Default::default(),
            },
        }
    }
}

impl VisibleCellData {
    fn update(&mut self, world: &World, coord: ICoord) {
        let layers = world.spatial_table.layers_at_checked(coord);
        self.tiles = layers.map(|&entity| {
            entity
                .map(|entity| {
                    let tile = world.components.tile.get(entity).cloned();
                    let colour_hint = world.components.colour_hint.get(entity).cloned();
                    let health = world.components.health.get(entity).cloned();
                    VisibleEntity {
                        tile,
                        colour_hint,
                        health,
                    }
                })
                .unwrap_or_default()
        });
    }
}

impl VisibleWorld for World {
    type VisionDistance = Circle;

    fn size(&self) -> UCoord {
        self.spatial_table.grid_size()
    }

    fn get_opacity(&self, coord: ICoord) -> u8 {
        Self::get_opacity(self, coord)
    }

    fn for_each_light_by_coord<F: FnMut(ICoord, &Light<Self::VisionDistance>)>(&self, mut f: F) {
        for (entity, light) in self.components.light.iter() {
            if let Some(coord) = self.spatial_table.coord_of(entity) {
                f(coord, light);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionError {
    InvalidMove,
    MoveOutOfBounds,
    NothingToGet,
    InventoryIsFull,
    CarIsOutOfFuel,
    TooTiredToDrive,
    CantDrive,
    CantStop,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    world: World,
    visibility_grid: VisibilityGrid<VisibleCellData>,
    agents: ComponentTable<Agent>,
    rng: Isaac64Rng,
    animation_rng: Isaac64Rng,
    player_entity: Entity,
    message_log: Vec<Message>,
    ai_context: AiContext,
    omniscient: bool,
    external_events: Vec<ExternalEvent>,
    turn_count: u64,
    game_over: bool,
    mode: Mode,
    time_of_day: TimeOfDay,
    terrain_type: TerrainType,
    energy_countdown: Countdown,
    pass_out_countdown: Countdown,
    passed_out_for: u32,
    walking_food_countdown: Countdown,
    driving_food_countdown: Countdown,
    distance_travelled: u32,
    distance_remaining: u32,
}

impl Game {
    pub fn new<R: Rng>(config: &Config, base_rng: &mut R) -> Self {
        let mut rng = Isaac64Rng::seed_from_u64(base_rng.random());
        let animation_rng = Isaac64Rng::seed_from_u64(base_rng.random());
        let terrain = Terrain::generate_end(&mut rng);
        let mut world = terrain.world;
        let visibility_grid = VisibilityGrid::new(world.spatial_table.grid_size());
        let player_spawn = terrain.player_spawn;
        let player_data = world::spawn::make_player();
        let player_location = Location {
            coord: player_spawn,
            layer: Some(Layer::Character),
        };
        let player_entity = world.insert_entity_data(player_location, player_data);
        let mut game = Self {
            ai_context: AiContext::new(world.size()),
            world,
            visibility_grid,
            agents: Default::default(),
            rng,
            animation_rng,
            player_entity,
            message_log: Vec::new(),
            omniscient: config.omniscient.is_some(),
            external_events: Default::default(),
            turn_count: 0,
            game_over: false,
            mode: Mode::Driving,
            time_of_day: TimeOfDay { hour: 6, minute: 0 },
            terrain_type: TerrainType::PinePlantation,
            energy_countdown: Countdown::new(120),
            pass_out_countdown: Countdown::new(30),
            passed_out_for: 0,
            walking_food_countdown: Countdown::new(60),
            driving_food_countdown: Countdown::new(2),
            distance_travelled: 0,
            distance_remaining: 2000,
        };
        game.systems();
        game.update_visibility();
        game
    }

    pub fn distance_remaining(&self) -> u32 {
        self.distance_remaining
    }

    pub fn passed_out_for(&self) -> u32 {
        self.passed_out_for
    }

    pub fn regenerate_terrain(&mut self) {
        let terrain = if self.at_end() {
            Terrain::generate_end(&mut self.rng)
        } else {
            Terrain::generate_pine_plantation(&mut self.rng)
        };
        let player_data = self.world.components.remove_entity_data(self.player_entity);
        self.world = terrain.world;
        let player_location = Location {
            coord: terrain.player_spawn,
            layer: Some(Layer::Character),
        };
        self.player_entity = self.world.insert_entity_data(player_location, player_data);
        self.visibility_grid = VisibilityGrid::new(self.world.spatial_table.grid_size());
    }

    pub fn terrain_type(&self) -> TerrainType {
        if self.at_start() {
            TerrainType::Start
        } else if self.at_end() {
            TerrainType::End
        } else {
            self.terrain_type
        }
    }

    pub fn time_of_day(&self) -> TimeOfDay {
        self.time_of_day
    }

    pub fn at_start(&self) -> bool {
        self.distance_travelled == 0
    }

    pub fn at_end(&self) -> bool {
        self.distance_remaining == 0
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn message_log(&self) -> &[Message] {
        &self.message_log
    }

    pub fn update_visibility(&mut self) {
        let update_fn = |data: &mut VisibleCellData, coord| {
            data.update(&self.world, coord);
        };
        if self.omniscient {
            self.visibility_grid.update_omniscient_custom(
                Rgb24::new_grey(255),
                &self.world,
                update_fn,
            );
        } else {
            let distance = if self.time_of_day.is_night() {
                Circle::new_squared(40)
            } else {
                Circle::new_squared(300)
            };
            self.visibility_grid.update_custom(
                Rgb24::new_grey(0),
                &self.world,
                distance,
                self.player_coord(),
                update_fn,
            );
        }
    }

    pub fn cell_visibility_at_coord(&self, coord: ICoord) -> CellVisibility<&VisibleCellData> {
        self.visibility_grid.get_visibility(coord)
    }

    /// Returns the coordinate of the player character
    pub fn player_coord(&self) -> ICoord {
        self.world
            .spatial_table
            .coord_of(self.player_entity)
            .expect("player does not have coord")
    }

    fn open_door(&mut self, entity: Entity) {
        self.world.components.apply_entity_update(
            entity,
            entity_update! {
                door_state: Some(DoorState::Open),
                tile: Some(Tile::DoorOpen),
                solid: None,
                opacity: None,
            },
        );
    }

    fn open_door_entity_adjacent_to_coord(
        &self,
        coord: ICoord,
        dest_coord: ICoord,
    ) -> Option<Entity> {
        for direction in Direction::all() {
            let potential_door_coord = coord + direction.coord();
            let delta = dest_coord - potential_door_coord;
            if delta.x.abs() <= 1 && delta.y.abs() <= 1 {
                if let Some(&Layers {
                    feature: Some(feature_entity),
                    character: None,
                    ..
                }) = self.world.spatial_table.layers_at(potential_door_coord)
                {
                    if let Some(DoorState::Open) =
                        self.world.components.door_state.get(feature_entity)
                    {
                        return Some(feature_entity);
                    }
                }
            }
        }
        None
    }

    fn close_door(&mut self, entity: Entity) {
        self.world.components.insert_entity_data(
            entity,
            entity_data! {
                door_state: DoorState::Closed,
                tile: Tile::DoorClosed,
                solid: (),
                opacity: 255,
            },
        );
    }

    fn player_walk(
        &mut self,
        direction: CardinalDirection,
    ) -> Result<Option<GameControlFlow>, ActionError> {
        let player_coord = self.player_coord();
        let new_player_coord = player_coord + direction.coord();
        if !new_player_coord.is_valid(self.world.size()) {
            // player would walk outside bounds of map
            return Err(ActionError::MoveOutOfBounds);
        }
        if let Some(layers) = self.world.spatial_table.layers_at(new_player_coord) {
            if let Some(feature_entity) = layers.feature {
                // If the player bumps into a door, open the door
                if let Some(DoorState::Closed) =
                    self.world.components.door_state.get(feature_entity)
                {
                    self.open_door(feature_entity);
                    self.message_log.push(Message::OpenDoor);
                    return Ok(None);
                }
                if self.world.components.your_car.contains(feature_entity) {
                    let choices = (0..self
                        .world
                        .components
                        .car_inventory
                        .get(self.player_entity)
                        .unwrap()
                        .size())
                        .map(|i| MenuChoice::TakeItemFromCar {
                            car_inventory_slot_index: i,
                        })
                        .collect();

                    match direction {
                        CardinalDirection::North => {
                            let car_inventory = Menu {
                                text: "Car Inventory\n\nChoose item to take, or free slot to move item into: (escape to cancel)".to_string(),
                                choices,
                            };
                            return Ok(Some(GameControlFlow::Menu(car_inventory)));
                        }
                        CardinalDirection::South => (),
                        CardinalDirection::East | CardinalDirection::West => {
                            if self
                                .world
                                .components
                                .car_fuel
                                .get(self.player_entity)
                                .unwrap()
                                .is_empty()
                            {
                                return Err(ActionError::CarIsOutOfFuel);
                            }
                            if self
                                .world
                                .components
                                .energy
                                .get(self.player_entity)
                                .unwrap()
                                .is_empty()
                            {
                                return Err(ActionError::TooTiredToDrive);
                            }
                            self.mode = Mode::Driving;
                            self.message_log.push(Message::GetInCar);
                            return Ok(None);
                        }
                    }
                }
                // Don't let the player walk through solid entities
                if self.world.components.solid.contains(feature_entity) {
                    if let Some(open_door_entity) =
                        self.open_door_entity_adjacent_to_coord(player_coord, new_player_coord)
                    {
                        self.close_door(open_door_entity);
                        self.message_log.push(Message::CloseDoor);
                        return Ok(None);
                    }
                    return Err(ActionError::InvalidMove);
                }
            }
            if let Some(floor_entity) = layers.floor {
                if self.world.components.grass.contains(floor_entity) {
                    self.world
                        .components
                        .tile
                        .insert(floor_entity, Tile::CrushedGrass);
                }
            }
            if let Some(character_entity) = layers.character {
                if self.world.components.character.contains(character_entity) {
                    self.world.player_bump_combat(
                        character_entity,
                        &mut self.rng,
                        &mut self.external_events,
                        &mut self.message_log,
                    );
                } else if self.world.components.zombie.contains(character_entity) {
                    self.message_log.push(Message::KickZombieCorpse);
                    if self.rng.random::<f32>() < 0.25 {
                        self.message_log.push(Message::DestroyZombieCorpse);
                        self.world.remove_entity(character_entity);
                    }
                }
                return Ok(None);
            }
            self.world
                .spatial_table
                .update_coord(self.player_entity, new_player_coord)
                .unwrap();
        }
        Ok(None)
    }

    fn npc_walk(
        &mut self,
        entity: Entity,
        direction: CardinalDirection,
    ) -> Option<GameControlFlow> {
        let current_coord = self
            .world
            .entity_coord(entity)
            .expect("Entity tried to walk but it doesn't have a coord");
        let new_coord = current_coord + direction.coord();
        if !new_coord.is_valid(self.world.size()) {
            // would walk outside bounds of map
            return None;
        }
        let npc = self.world.components.npc.get(entity).unwrap();
        if let Some(&Layers {
            feature,
            character,
            floor,
            ..
        }) = self.world.spatial_table.layers_at(new_coord)
        {
            if let Some(feature_entity) = feature {
                // If the npc bumps into a door, open the door
                if let Some(DoorState::Closed) =
                    self.world.components.door_state.get(feature_entity)
                {
                    if npc.movement.can_open_doors {
                        self.open_door(feature_entity);
                        return None;
                    }
                }
                if self.world.components.difficult.contains(feature_entity) {
                    if !npc.movement.can_traverse_difficult {
                        return None;
                    }
                }
            }
            // Don't let them walk into other characters
            if let Some(character_entity) = character {
                if self.world.components.player.contains(character_entity) {
                    let damage_range = self
                        .world
                        .components
                        .bump_damage
                        .get(entity)
                        .cloned()
                        .unwrap_or(1..=1);
                    self.world.damage_player(
                        entity,
                        self.rng.random_range(damage_range),
                        &mut self.rng,
                        &mut self.external_events,
                        &mut self.message_log,
                    );
                }
                return None;
            }
            if let Some(floor) = floor {
                if self.world.components.grass.contains(floor) {
                    self.world.components.tile.insert(floor, Tile::CrushedGrass);
                }
            }
        }
        if !self
            .world
            .can_npc_traverse_feature_at_coord_with_entity(new_coord, entity)
        {
            return None;
        }
        self.world
            .spatial_table
            .update_coord(entity, new_coord)
            .unwrap();
        None
    }

    // Create agents for npcs that lack agents and remove agents for agents whose npcs have been
    // removed.
    fn npc_setup_agents(&mut self) {
        for entity in self.world.components.npc.entities() {
            if !self.agents.contains(entity) {
                self.agents.insert(entity, Agent::new(self.world.size()));
            }
        }
        let mut agents_to_remove = Vec::new();
        for entity in self.agents.entities() {
            if !self.world.components.npc.contains(entity) {
                agents_to_remove.push(entity);
            }
        }
        for entity in agents_to_remove {
            self.agents.remove(entity);
        }
    }

    fn npc_turn(&mut self) -> Option<GameControlFlow> {
        self.npc_setup_agents();
        self.ai_context.update(self.player_entity, &self.world);
        let agent_entities = self.agents.entities().collect::<Vec<_>>();
        for agent_entity in agent_entities {
            if !self.world.components.character.contains(agent_entity) {
                // so that dead zombies don't get a turn
                continue;
            }
            if let Some(slow) = self.world.components.slow.get(agent_entity) {
                if !self.turn_count.is_multiple_of(*slow) {
                    continue;
                }
            }
            let ai_input = self.agents.get_mut(agent_entity).unwrap().act(
                agent_entity,
                &self.world,
                self.player_entity,
                &mut self.ai_context,
                &mut self.rng,
            );
            if let Some(input) = ai_input {
                match input {
                    Input::Wait => (),
                    Input::Walk(direction) => {
                        if let Some(control_flow) = self.npc_walk(agent_entity, direction) {
                            return Some(control_flow);
                        }
                    }
                    _ => (),
                }
            }
        }
        self.systems();
        self.passed_out_for = self.passed_out_for.saturating_sub(1);
        self.turn_count += 1;
        self.decrease_player_stats();
        if self.turn_count % 1 == 0 {
            self.time_of_day = self.time_of_day.add_minutes(1);
        }
        if self.win() {
            self.message_log.push(Message::MakeWish);
            self.update_visibility();
            return Some(GameControlFlow::Win);
        }
        self.check_game_over()
    }

    fn systems(&mut self) {
        self.world.handle_resurrection();
        self.world
            .handle_night_stalkers(self.time_of_day, &mut self.rng, &mut self.message_log);
    }

    fn decrease_player_stats(&mut self) {
        let energy = self
            .world
            .components
            .energy
            .get_mut(self.player_entity)
            .unwrap();
        match self.mode {
            Mode::Walking => {
                if self.passed_out_for == 0 {
                    if energy.is_empty() {
                        if self.pass_out_countdown.value == 5 {
                            self.message_log.push(Message::AboutToPassOut);
                        }
                        if self.pass_out_countdown.tick() {
                            self.message_log.push(Message::PassOut);
                            self.passed_out_for = 30;
                        }
                    } else {
                        self.pass_out_countdown.reset();
                        if self.energy_countdown.tick() {
                            energy.decrease(1);
                        }
                    }
                }
                if self.passed_out_for == 1 {
                    energy.set_current(1);
                    self.pass_out_countdown.reset();
                }
            }
            Mode::Driving => {
                if !energy.is_empty() {
                    energy.decrease(1);
                }
                if energy.is_empty() {
                    self.message_log.push(Message::TooTiredToDrive);
                    self.message_log.push(Message::GetOutOfCar);
                    self.mode = Mode::Walking;
                }
            }
        }
        let food = self
            .world
            .components
            .food
            .get_mut(self.player_entity)
            .unwrap();
        match self.mode {
            Mode::Walking => {
                if self.walking_food_countdown.tick() {
                    if food.is_empty() {
                        self.message_log.push(Message::DamageFromHunger);
                        self.world
                            .components
                            .health
                            .get_mut(self.player_entity)
                            .unwrap()
                            .decrease(1);
                    } else {
                        food.decrease(1);
                    }
                }
            }
            Mode::Driving => {
                if self.driving_food_countdown.tick() {
                    if food.is_empty() {
                        self.message_log.push(Message::DamageFromHunger);
                        self.world
                            .components
                            .health
                            .get_mut(self.player_entity)
                            .unwrap()
                            .decrease(1);
                    } else {
                        food.decrease(1);
                    }
                }
            }
        }
    }

    fn check_game_over(&mut self) -> Option<GameControlFlow> {
        if self.game_over {
            return Some(GameControlFlow::GameOver(GameOverReason::YouDied));
        }
        if self.world.is_game_over() {
            self.game_over = true;
            self.world
                .components
                .tile
                .insert(self.player_entity, Tile::DeadPlayer);
            self.player_drop_all_items();
            self.update_visibility();
            self.message_log.push(Message::YouDie);
            self.external_events.push(ExternalEvent::Death);
            return Some(GameControlFlow::GameOver(GameOverReason::YouDied));
        }
        None
    }

    fn win(&self) -> bool {
        if let Some(layers) = self.world.spatial_table.layers_at(self.player_coord()) {
            if let Some(feature) = layers.feature {
                if self.world.components.typewriter.contains(feature) {
                    return true;
                }
            }
        }
        false
    }

    fn cleanup(&mut self) {
        let to_remove = self
            .world
            .components
            .to_remove
            .entities()
            .collect::<Vec<_>>();
        for entity in to_remove {
            self.world.remove_entity(entity);
        }
    }

    #[must_use]
    pub(crate) fn handle_tick(
        &mut self,
        _since_last_tick: Duration,
        _config: &Config,
    ) -> Option<GameControlFlow> {
        let initially_blockd = self.is_gameplay_blocked();
        if initially_blockd && !self.is_gameplay_blocked() {
            let result = self.npc_turn();
            if result.is_some() {
                return result;
            }
        }
        self.cleanup();
        self.update_visibility();
        self.check_game_over()
    }

    fn pass_time(&mut self) {}

    pub fn is_gameplay_blocked(&self) -> bool {
        self.passed_out_for > 0
    }

    #[must_use]
    pub(crate) fn handle_input(
        &mut self,
        input: Input,
    ) -> Result<Option<GameControlFlow>, ActionError> {
        let game_control_flow = match input {
            Input::Walk(direction) => {
                let result = self.player_walk(direction);
                match result {
                    Ok(x) => x,
                    Err(action_error) => {
                        self.message_log.push(Message::ActionError(action_error));
                        return Err(action_error);
                    }
                }
            }
            Input::Wait => {
                if self.passed_out_for == 0 {
                    self.message_log.push(Message::Wait);
                }
                self.pass_time();
                None
            }
            Input::Get => {
                if let Err(e) = self.player_get_item() {
                    self.message_log.push(Message::ActionError(e));
                    return Err(e);
                }
                None
            }
            Input::ContinueDriving => {
                if self.at_end() {
                    return Err(ActionError::CantDrive);
                }
                self.time_of_day = self.time_of_day.add_minutes(59);
                let fuel = self
                    .world
                    .components
                    .car_fuel
                    .get_mut(self.player_entity)
                    .unwrap();
                self.distance_travelled += 1;
                self.distance_remaining = self
                    .distance_remaining
                    .saturating_sub(self.rng.random_range(20..50));
                fuel.decrease(1);
                if fuel.is_empty() {
                    self.mode = Mode::Walking;
                    self.message_log.push(Message::OutOfFuel);
                    self.message_log.push(Message::GetOutOfCar);
                }
                self.regenerate_terrain();
                None
            }
            Input::StopDriving => {
                if self.at_start() {
                    return Err(ActionError::CantStop);
                }
                self.mode = Mode::Walking;
                self.message_log.push(Message::GetOutOfCar);
                None
            }
        };
        if game_control_flow.is_some() {
            return Ok(game_control_flow);
        }
        let game_control_flow = self.npc_turn();
        if game_control_flow.is_some() {
            return Ok(game_control_flow);
        }
        self.update_visibility();
        Ok(self.check_game_over())
    }

    fn player_inventory(&self) -> &Inventory {
        self.world
            .components
            .inventory
            .get(self.player_entity)
            .unwrap()
    }

    fn player_inventory_mut(&mut self) -> &mut Inventory {
        self.world
            .components
            .inventory
            .get_mut(self.player_entity)
            .unwrap()
    }

    fn car_inventory(&self) -> &Inventory {
        self.world
            .components
            .car_inventory
            .get(self.player_entity)
            .unwrap()
    }

    fn car_inventory_mut(&mut self) -> &mut Inventory {
        self.world
            .components
            .car_inventory
            .get_mut(self.player_entity)
            .unwrap()
    }

    fn player_get_item(&mut self) -> Result<(), ActionError> {
        let player_coord = self.player_coord();
        let layers = self.world.spatial_table.layers_at_checked(player_coord);
        if let Some(item_entity) = layers.item {
            if let Some(&item) = self.world.components.item.get(item_entity) {
                if self.player_inventory().has_free_slot() {
                    self.world.spatial_table.remove(item_entity);
                    let entity_data = self.world.components.remove_entity_data(item_entity);
                    let inventory = self.player_inventory_mut();
                    let slot = inventory.first_free_slot().unwrap();
                    *slot = Some(entity_data);
                    self.message_log.push(Message::GetItem(item));
                } else {
                    return Err(ActionError::InventoryIsFull);
                }
            }
            Ok(())
        } else {
            Err(ActionError::NothingToGet)
        }
    }

    pub(crate) fn handle_choice(&mut self, choice: MenuChoice) -> Option<GameControlFlow> {
        match choice {
            MenuChoice::Empty => (),
            MenuChoice::DropItem(i) => self.player_drop_item(i),
            MenuChoice::ApplyItem(i) => {
                if let Some(control_flow) = self.player_apply_item(i) {
                    return Some(control_flow);
                }
            }
            MenuChoice::TakeItemFromCar {
                car_inventory_slot_index,
            } => {
                let item = self.car_inventory().get(car_inventory_slot_index);
                match item {
                    Some(_) => {
                        let player_inventory = self.player_inventory();
                        if player_inventory.has_free_slot() {
                            let car_inventory = self.car_inventory_mut();
                            let item_data = car_inventory.remove(car_inventory_slot_index).unwrap();
                            self.message_log
                                .push(Message::TransferItemFromCar(item_data.item.unwrap()));
                            *self.player_inventory_mut().first_free_slot().unwrap() =
                                Some(item_data);
                        }
                    }
                    None => {
                        let choices = (0..self
                            .world
                            .components
                            .inventory
                            .get(self.player_entity)
                            .unwrap()
                            .size())
                            .map(|i| MenuChoice::TransferItemToCar {
                                car_inventory_slot_index,
                                player_inventor_slot_index: i,
                            })
                            .collect();
                        return Some(GameControlFlow::Menu(Menu {
                            text: "Choose item from your inventory to move into car: (escape to cancel)".to_string(),
                            choices,
                        }));
                    }
                }
            }
            MenuChoice::TransferItemToCar {
                car_inventory_slot_index,
                player_inventor_slot_index,
            } => {
                let player_inventory = self.player_inventory_mut();
                if let Some(item_data) = player_inventory.remove(player_inventor_slot_index) {
                    self.message_log
                        .push(Message::TransferItemToCar(item_data.item.unwrap()));
                    let car_inventory = self.car_inventory_mut();
                    car_inventory.insert(car_inventory_slot_index, item_data);
                }
            }
        }
        self.npc_turn();
        self.check_game_over()
    }

    fn is_player_next_to_car(&self) -> bool {
        let player_coord = self.player_coord();
        for d in CardinalDirections {
            let c = player_coord + d.coord();
            if let Some(layers) = self.world.spatial_table.layers_at(c) {
                if let Some(feature) = layers.feature {
                    if self.world.components.your_car.contains(feature) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn player_apply_item(&mut self, i: usize) -> Option<GameControlFlow> {
        let player_next_to_car = self.is_player_next_to_car();
        let inventory = self
            .world
            .components
            .inventory
            .get_mut(self.player_entity)
            .unwrap();
        let mut despawn_night_stalkers = false;
        if let Some(item_data) = inventory.get(i) {
            if let Some(item) = item_data.item {
                match item {
                    Item::MedKit => {
                        self.message_log.push(Message::ApplyItem(item));
                        self.world
                            .components
                            .health
                            .get_mut(self.player_entity)
                            .unwrap()
                            .increase(10);
                    }
                    Item::Fruit => {
                        self.message_log.push(Message::ApplyItem(item));
                        self.world
                            .components
                            .food
                            .get_mut(self.player_entity)
                            .unwrap()
                            .increase(5);
                    }
                    Item::Food => {
                        self.message_log.push(Message::ApplyItem(item));
                        self.world
                            .components
                            .food
                            .get_mut(self.player_entity)
                            .unwrap()
                            .increase(10);
                    }
                    Item::Firewood => {
                        if !self.time_of_day.is_night() {
                            self.message_log.push(Message::CanOnlySleepAtNight);
                            return None;
                        }
                        self.message_log.push(Message::ApplyItem(item));
                        self.time_of_day = self.time_of_day.add_hours(2);
                        despawn_night_stalkers = true;
                        self.world
                            .components
                            .energy
                            .get_mut(self.player_entity)
                            .unwrap()
                            .increase(5);
                        self.world
                            .components
                            .food
                            .get_mut(self.player_entity)
                            .unwrap()
                            .decrease(2);
                    }
                    Item::Coffee => {
                        self.message_log.push(Message::ApplyItem(item));
                        self.world
                            .components
                            .energy
                            .get_mut(self.player_entity)
                            .unwrap()
                            .increase(10);
                    }
                    Item::FuelCan => {
                        if player_next_to_car {
                            self.message_log.push(Message::ApplyItem(item));
                            self.world
                                .components
                                .car_fuel
                                .get_mut(self.player_entity)
                                .unwrap()
                                .increase(10);
                        } else {
                            self.message_log.push(Message::MustBeNextToCarToRefuel);
                            return None;
                        }
                    }
                }
                inventory.remove(i);
            }
        }
        if despawn_night_stalkers {
            self.world.despawn_night_stalkers(&mut self.message_log);
        }
        None
    }

    fn player_drop_all_items(&mut self) {
        let inventory = self
            .world
            .components
            .inventory
            .get(self.player_entity)
            .unwrap();
        let mut items = vec![];
        for i in 0..inventory.size() {
            let inventory = self
                .world
                .components
                .inventory
                .get_mut(self.player_entity)
                .unwrap();
            if let Some(item_data) = inventory.remove(i) {
                items.push(item_data);
            }
        }
        for item_data in items {
            if let Some(coord) = self.world.nearest_itemless_coord(self.player_coord()) {
                self.world.spawn_entity(
                    Location {
                        coord,
                        layer: Some(Layer::Item),
                    },
                    item_data,
                );
            }
        }
    }

    fn player_drop_item(&mut self, i: usize) {
        let inventory = self
            .world
            .components
            .inventory
            .get_mut(self.player_entity)
            .unwrap();
        if let Some(item_data) = inventory.remove(i) {
            if let Some(item) = item_data.item {
                self.message_log.push(Message::DropItem(item));
            }
            if let Some(coord) = self.world.nearest_itemless_coord(self.player_coord()) {
                self.world.spawn_entity(
                    Location {
                        coord,
                        layer: Some(Layer::Item),
                    },
                    item_data,
                );
            }
        }
    }

    pub fn world_size(&self) -> UCoord {
        self.world.spatial_table.grid_size()
    }

    pub fn take_external_events(&mut self) -> Vec<ExternalEvent> {
        std::mem::take(&mut self.external_events)
    }

    pub fn player_stats(&self) -> PlayerStats {
        PlayerStats {
            health: *self
                .world
                .components
                .health
                .get(self.player_entity)
                .unwrap(),
            energy: *self
                .world
                .components
                .energy
                .get(self.player_entity)
                .unwrap(),
            car_fuel: *self
                .world
                .components
                .car_fuel
                .get(self.player_entity)
                .unwrap(),
            food: *self.world.components.food.get(self.player_entity).unwrap(),
        }
    }

    pub fn inventory_size(&self) -> usize {
        self.world
            .components
            .inventory
            .get(self.player_entity)
            .unwrap()
            .size()
    }

    pub fn inventory_item(&self, i: usize) -> Option<Item> {
        self.player_inventory()
            .get(i)
            .map(|entity_data| entity_data.item.unwrap())
    }

    pub fn car_inventory_item(&self, i: usize) -> Option<Item> {
        self.world
            .components
            .car_inventory
            .get(self.player_entity)
            .unwrap()
            .get(i)
            .map(|entity_data| entity_data.item.unwrap())
    }
}
