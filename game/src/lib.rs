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
    pub fn add_hours(&self, hours: u32) -> Self {
        Self {
            hour: ((self.hour as u32 + hours) % 24) as u8,
            minute: self.minute,
        }
    }
    pub fn period(&self) -> PeriodOfDay {
        use PeriodOfDay::*;
        match self.hour {
            0..=4 => Night,
            5 => Dawn,
            6..=11 => Morning,
            12..=17 => Afternoon,
            18 => Dusk,
            19.. => Night,
        }
    }
    pub fn is_night(&self) -> bool {
        match self.period() {
            PeriodOfDay::Night => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}:{:02} ({})", self.hour, self.minute, self.period())
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
    YouDie,
    NpcDies(NpcType),
    OpenDoor,
    CloseDoor,
    ActionError(ActionError),
    GetItem(Item),
    DropItem(Item),
    GetOutOfCar,
    GetInCar,
}

#[derive(Debug, Clone, Copy)]
pub enum MenuChoice {
    Empty,
    DropItem(usize),
    ApplyItem(usize),
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

#[derive(Debug, Clone, Copy)]
pub enum Win {
    Good,
    Bad,
}

#[derive(Debug)]
pub enum GameControlFlow {
    GameOver(GameOverReason),
    Win(Win),
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
    Forest,
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
    NothingToGet,
    InventoryIsFull,
}

#[derive(Serialize, Deserialize)]
struct Level {
    world: World,
    visibility_grid: VisibilityGrid<VisibleCellData>,
    agents: ComponentTable<Agent>,
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
}

impl Game {
    pub fn new<R: Rng>(config: &Config, base_rng: &mut R) -> Self {
        let mut rng = Isaac64Rng::seed_from_u64(base_rng.random());
        let animation_rng = Isaac64Rng::seed_from_u64(base_rng.random());
        let terrain = Terrain::generate_text();
        let mut world = terrain.world;
        let visibility_grid = VisibilityGrid::new(world.spatial_table.grid_size());
        let player_spawn = ICoord::new(0, 0);
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
            terrain_type: TerrainType::Forest,
        };
        game.systems();
        game.update_visibility();
        game
    }

    pub fn terrain_type(&self) -> TerrainType {
        self.terrain_type
    }

    pub fn time_of_day(&self) -> TimeOfDay {
        self.time_of_day
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
            let distance = Circle::new_squared(300);
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
            return Err(ActionError::InvalidMove);
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
                if self.world.components.your_car.contains(feature_entity) {
                    self.mode = Mode::Driving;
                    self.message_log.push(Message::GetInCar);
                    return Ok(None);
                }
            }
            if let Some(character_entity) = layers.character {
                self.world.player_bump_combat(
                    character_entity,
                    &mut self.rng,
                    &mut self.external_events,
                    &mut self.message_log,
                );
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
        if let Some(&Layers {
            feature, character, ..
        }) = self.world.spatial_table.layers_at(new_coord)
        {
            if let Some(feature_entity) = feature {
                // If the npc bumps into a door, open the door
                if let Some(DoorState::Closed) =
                    self.world.components.door_state.get(feature_entity)
                {
                    self.open_door(feature_entity);
                    return None;
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
        self.turn_count += 1;
        if let Some(win) = self.win() {
            self.update_visibility();
            return Some(GameControlFlow::Win(win));
        }
        self.check_game_over()
    }

    fn systems(&mut self) {}

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
            Some(GameControlFlow::GameOver(GameOverReason::YouDied))
        } else {
            None
        }
    }

    fn win(&self) -> Option<Win> {
        None
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
        false
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
                self.message_log.push(Message::Wait);
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
                self.time_of_day = self.time_of_day.add_hours(1);
                None
            }
            Input::StopDriving => {
                self.mode = Mode::Walking;
                self.message_log.push(Message::GetOutOfCar);
                None
            }
        };
        if game_control_flow.is_some() {
            return Ok(game_control_flow);
        }
        if !self.is_gameplay_blocked() {
            let game_control_flow = self.npc_turn();
            if game_control_flow.is_some() {
                return Ok(game_control_flow);
            }
        }
        self.update_visibility();
        Ok(self.check_game_over())
    }

    fn player_get_item(&mut self) -> Result<(), ActionError> {
        let player_coord = self.player_coord();
        let layers = self.world.spatial_table.layers_at_checked(player_coord);
        if let Some(item_entity) = layers.item {
            if let Some(&item) = self.world.components.item.get(item_entity) {
                let inventry = self
                    .world
                    .components
                    .inventory
                    .get_mut(self.player_entity)
                    .unwrap();
                if let Some(slot) = inventry.first_free_slot() {
                    *slot = Some(item_entity);
                    self.world.spatial_table.remove(item_entity);
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
        }
        self.npc_turn();
        self.check_game_over()
    }

    fn player_apply_item(&mut self, i: usize) -> Option<GameControlFlow> {
        let inventory = self
            .world
            .components
            .inventory
            .get_mut(self.player_entity)
            .unwrap();
        if let Some(item_entity) = inventory.get(i) {
            if let Some(&item) = self.world.components.item.get(item_entity) {
                match item {
                    Item::MedKit => (),
                }
            }
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
        for i in 0..inventory.size() {
            let inventory = self
                .world
                .components
                .inventory
                .get_mut(self.player_entity)
                .unwrap();
            if let Some(item_entity) = inventory.remove(i) {
                if let Some(coord) = self.world.nearest_itemless_coord(self.player_coord()) {
                    let _ = self.world.spatial_table.update(
                        item_entity,
                        Location {
                            coord,
                            layer: Some(Layer::Item),
                        },
                    );
                }
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
        if let Some(item_entity) = inventory.remove(i) {
            if let Some(&item) = self.world.components.item.get(item_entity) {
                self.message_log.push(Message::DropItem(item));
            }
            if let Some(coord) = self.world.nearest_itemless_coord(self.player_coord()) {
                let _ = self.world.spatial_table.update(
                    item_entity,
                    Location {
                        coord,
                        layer: Some(Layer::Item),
                    },
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
        let inventory = self
            .world
            .components
            .inventory
            .get(self.player_entity)
            .unwrap();
        inventory
            .get(i)
            .map(|entity| *self.world.components.item.get(entity).unwrap())
    }
}
