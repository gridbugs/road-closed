use crate::{
    colours,
    controls::{AppInput, AppInputDriving, Controls},
    game_instance::{GameInstance, GameInstanceStorable, item_string_for_menu, message_to_text},
    text,
};
use chargrid::{self, border::BorderStyle, control_flow::*, menu, prelude::*};
use game::{
    Config as GameConfig, GameOverReason, Item, Menu as GameMenu, MenuChoice as GameMenuChoice,
    Mode, Victory,
    witness::{self, Witness},
};
use general_storage_static::{self as storage, StaticStorage as Storage, format};
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    sfx_volume: f32,
    won: bool,
    first_run: bool,
    victories: Vec<Victory>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sfx_volume: 0.5,
            won: false,
            first_run: true,
            victories: Vec::new(),
        }
    }
}

/// An interactive, renderable process yielding a value of type `T`
pub type AppCF<T> = CF<Option<T>, GameLoopData>;
pub type State = GameLoopData;

const MENU_FADE_SPEC: menu::identifier::fade_spec::FadeSpec = {
    use menu::identifier::fade_spec::*;
    FadeSpec {
        on_select: Fade {
            to: To {
                rgba32: Layers {
                    foreground: colours::MENU_SELECT_FOREGROUND.to_rgba32(255),
                    background: colours::MENU_SELECT_BACKGROUND.to_rgba32(255),
                },
                bold: true,
                underline: false,
            },
            from: From::current(),
            durations: Layers {
                foreground: Duration::from_millis(128),
                background: Duration::from_millis(128),
            },
        },
        on_deselect: Fade {
            to: To {
                rgba32: Layers {
                    foreground: colours::MENU_FOREGROUND.to_rgba32(255),
                    background: colours::MENU_BACKGROUND.to_rgba32(255),
                },
                bold: false,
                underline: false,
            },
            from: From::current(),
            durations: Layers {
                foreground: Duration::from_millis(128),
                background: Duration::from_millis(128),
            },
        },
    }
};

pub enum InitialRngSeed {
    U64(u64),
    Random,
}

struct RngSeedSource {
    next_seed: u64,
    seed_rng: Isaac64Rng,
}

impl RngSeedSource {
    fn new(initial_rng_seed: InitialRngSeed) -> Self {
        let mut seed_rng = Isaac64Rng::from_rng(&mut rand::rng());
        let next_seed = match initial_rng_seed {
            InitialRngSeed::U64(seed) => seed,
            InitialRngSeed::Random => seed_rng.random(),
        };
        Self {
            next_seed,
            seed_rng,
        }
    }

    fn next_seed(&mut self) -> u64 {
        let seed = self.next_seed;
        self.next_seed = self.seed_rng.random();
        #[cfg(feature = "print_stdout")]
        println!("RNG Seed: {}", seed);
        #[cfg(feature = "print_log")]
        log::info!("RNG Seed: {}", seed);
        seed
    }
}

pub struct AppStorage {
    pub handle: Storage,
    pub save_game_key: String,
    pub config_key: String,
    pub controls_key: String,
}

impl AppStorage {
    const SAVE_GAME_STORAGE_FORMAT: format::Bincode = format::Bincode;
    const CONFIG_STORAGE_FORMAT: format::JsonPretty = format::JsonPretty;
    const CONTROLS_STORAGE_FORMAT: format::JsonPretty = format::JsonPretty;

    fn save_game(&mut self, instance: &GameInstanceStorable) {
        let result = self.handle.store(
            &self.save_game_key,
            &instance,
            Self::SAVE_GAME_STORAGE_FORMAT,
        );
        if let Err(e) = result {
            use storage::{StoreError, StoreRawError};
            match e {
                StoreError::FormatError(e) => log::error!("Failed to format save file: {}", e),
                StoreError::Raw(e) => match e {
                    StoreRawError::IoError(e) => {
                        log::error!("Error while writing save data: {}", e)
                    }
                },
            }
        }
    }

    fn load_game(&self) -> Option<GameInstanceStorable> {
        let result = self.handle.load::<_, GameInstanceStorable, _>(
            &self.save_game_key,
            Self::SAVE_GAME_STORAGE_FORMAT,
        );
        match result {
            Err(e) => {
                use storage::{LoadError, LoadRawError};
                match e {
                    LoadError::FormatError(e) => log::error!("Failed to parse save file: {}", e),
                    LoadError::Raw(e) => match e {
                        LoadRawError::IoError(e) => {
                            log::error!("Error while reading save data: {}", e)
                        }
                        LoadRawError::NoSuchKey => (),
                    },
                }
                None
            }
            Ok(instance) => Some(instance),
        }
    }

    fn clear_game(&mut self) {
        if self.handle.exists(&self.save_game_key) {
            if let Err(e) = self.handle.remove(&self.save_game_key) {
                use storage::RemoveError;
                match e {
                    RemoveError::IoError(e) => {
                        log::error!("Error while removing data: {}", e)
                    }
                    RemoveError::NoSuchKey => (),
                }
            }
        }
    }

    fn save_config(&mut self, config: &Config) {
        let result = self
            .handle
            .store(&self.config_key, &config, Self::CONFIG_STORAGE_FORMAT);
        if let Err(e) = result {
            use storage::{StoreError, StoreRawError};
            match e {
                StoreError::FormatError(e) => log::error!("Failed to format config: {}", e),
                StoreError::Raw(e) => match e {
                    StoreRawError::IoError(e) => {
                        log::error!("Error while writing config: {}", e)
                    }
                },
            }
        }
    }

    fn load_config(&self) -> Option<Config> {
        let result = self
            .handle
            .load::<_, Config, _>(&self.config_key, Self::CONFIG_STORAGE_FORMAT);
        match result {
            Err(e) => {
                use storage::{LoadError, LoadRawError};
                match e {
                    LoadError::FormatError(e) => log::error!("Failed to parse config file: {}", e),
                    LoadError::Raw(e) => match e {
                        LoadRawError::IoError(e) => {
                            log::error!("Error while reading config: {}", e)
                        }
                        LoadRawError::NoSuchKey => (),
                    },
                }
                None
            }
            Ok(instance) => Some(instance),
        }
    }

    fn save_controls(&mut self, controls: &Controls) {
        let result =
            self.handle
                .store(&self.controls_key, &controls, Self::CONTROLS_STORAGE_FORMAT);
        if let Err(e) = result {
            use storage::{StoreError, StoreRawError};
            match e {
                StoreError::FormatError(e) => log::error!("Failed to format controls: {}", e),
                StoreError::Raw(e) => match e {
                    StoreRawError::IoError(e) => {
                        log::error!("Error while writing controls: {}", e)
                    }
                },
            }
        }
    }

    fn load_controls(&self) -> Option<Controls> {
        let result = self
            .handle
            .load::<_, Controls, _>(&self.controls_key, Self::CONTROLS_STORAGE_FORMAT);
        match result {
            Err(e) => {
                use storage::{LoadError, LoadRawError};
                match e {
                    LoadError::FormatError(e) => {
                        log::error!("Failed to parse controls file: {}", e)
                    }
                    LoadError::Raw(e) => match e {
                        LoadRawError::IoError(e) => {
                            log::error!("Error while reading controls: {}", e)
                        }
                        LoadRawError::NoSuchKey => (),
                    },
                }
                None
            }
            Ok(instance) => Some(instance),
        }
    }
}

fn new_game(
    rng_seed_source: &mut RngSeedSource,
    game_config: &GameConfig,
) -> (GameInstance, witness::Running) {
    let mut rng = Isaac64Rng::seed_from_u64(rng_seed_source.next_seed());
    GameInstance::new(game_config, &mut rng)
}

#[derive(Clone, Copy)]
struct ScreenShake {
    countdown: u32,
    offset: ICoord,
}

pub struct GameLoopData {
    instance: Option<GameInstance>,
    controls: Controls,
    game_config: GameConfig,
    storage: AppStorage,
    rng_seed_source: RngSeedSource,
    config: Config,
    cursor: Option<ICoord>,
    screen_shake: Option<ScreenShake>,
}

impl GameLoopData {
    pub fn new(
        game_config: GameConfig,
        mut storage: AppStorage,
        initial_rng_seed: InitialRngSeed,
        force_new_game: bool,
    ) -> (Self, GameLoopState) {
        let mut rng_seed_source = RngSeedSource::new(initial_rng_seed);
        let config = storage.load_config().unwrap_or_default();
        let (instance, state) = match storage.load_game() {
            Some(instance) => {
                let (instance, running) = instance.into_game_instance();
                (
                    Some(instance),
                    GameLoopState::Playing(running.into_witness()),
                )
            }
            None => {
                if force_new_game {
                    let (instance, running) = new_game(&mut rng_seed_source, &game_config);
                    (
                        Some(instance),
                        GameLoopState::Playing(running.into_witness()),
                    )
                } else {
                    (None, GameLoopState::MainMenu)
                }
            }
        };
        let controls = if let Some(controls) = storage.load_controls() {
            controls
        } else {
            let controls = Controls::default();
            storage.save_controls(&controls);
            controls
        };
        (
            Self {
                instance,
                controls,
                game_config,
                storage,
                rng_seed_source,
                config,
                cursor: None,
                screen_shake: None,
            },
            state,
        )
    }

    // XXX the witness system is overly restrictive
    fn try_save_instance_cheat(&mut self) {
        if let Some(instance) = self.instance.take() {
            let instance = instance.into_storable(witness::Running::cheat());
            self.storage.save_game(&instance);
            let (instance, _running) = instance.into_game_instance();
            self.instance = Some(instance);
        }
    }

    fn save_instance(&mut self, running: witness::Running) -> witness::Running {
        let instance = self.instance.take().unwrap().into_storable(running);
        self.storage.save_game(&instance);
        let (instance, running) = instance.into_game_instance();
        self.instance = Some(instance);
        running
    }

    fn clear_saved_game(&mut self) {
        self.storage.clear_game();
        self.instance = None;
    }

    fn new_game(&mut self) -> witness::Running {
        let (instance, running) = new_game(&mut self.rng_seed_source, &self.game_config);
        self.instance = Some(instance);
        running
    }

    fn save_config(&mut self) {
        self.storage.save_config(&self.config);
    }

    fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        if let Some(instance) = self.instance.as_ref() {
            let offset = self
                .screen_shake
                .map(|s| s.offset)
                .unwrap_or(ICoord::new(0, 0));
            instance.render(ctx, fb, self.cursor, offset);
            let colour = colours::CURSOR.to_rgba32(187);
            if let Some(cursor) = self.cursor {
                let render_cell = RenderCell::default().with_background(colour);
                fb.set_cell_relative_to_ctx(ctx, cursor, 50, render_cell);
            }
        }
    }

    fn update(&mut self, event: Event, running: witness::Running) -> GameLoopState {
        let instance = self.instance.as_mut().unwrap();
        let witness = match event {
            Event::Input(input) => {
                self.cursor = None;
                match instance.game.inner_ref().mode() {
                    Mode::Driving => {
                        if let Some(app_input) = self.controls.get_driving(input) {
                            let (witness, _action_result) = match app_input {
                                AppInputDriving::ContinueDriving => {
                                    running.continue_driving(&mut instance.game)
                                }
                                AppInputDriving::StopDriving => {
                                    running.stop_driving(&mut instance.game)
                                }
                            };
                            witness
                        } else {
                            running.into_witness()
                        }
                    }
                    Mode::Walking => {
                        if let Some(app_input) = self.controls.get_walking(input) {
                            if instance.game.inner_ref().is_gameplay_blocked() {
                                running.into_witness()
                            } else {
                                let (witness, _action_result) = match app_input {
                                    AppInput::Direction(direction) => {
                                        let witness = running.walk(&mut instance.game, direction);
                                        witness
                                    }
                                    AppInput::Wait => running.wait(&mut instance.game),
                                    AppInput::Get => running.get(&mut instance.game),
                                    AppInput::MessageLog => {
                                        return GameLoopState::MessageLog(running);
                                    }
                                    AppInput::DropItem => (
                                        drop_menu_witness(instance.game.inner_ref(), running),
                                        Ok(()),
                                    ),
                                    AppInput::ApplyItem => (
                                        apply_menu_witness(instance.game.inner_ref(), running),
                                        Ok(()),
                                    ),
                                };
                                witness
                            }
                        } else {
                            if let Input::Mouse(MouseInput::MouseMove { coord, .. }) = input {
                                self.cursor = Some(coord);
                            }
                            if let Input::Mouse(MouseInput::MousePress { coord, .. }) = input {
                                self.cursor = Some(coord);
                            }
                            if let Input::Mouse(MouseInput::MouseRelease { coord, .. }) = input {
                                self.cursor = Some(coord);
                            }
                            if let Input::Keyboard(KeyboardInput::Char('?')) = input {
                                return GameLoopState::Help(running);
                            }
                            running.into_witness()
                        }
                    }
                }
            }
            Event::Tick(since_previous) => {
                let witness = running.tick(&mut instance.game, since_previous, &self.game_config);
                self.screen_shake = self.screen_shake.and_then(|mut screen_shake| {
                    if screen_shake.countdown == 0 {
                        None
                    } else {
                        screen_shake.countdown -= 1;
                        Some(screen_shake)
                    }
                });
                witness
            }
            _ => Witness::Running(running),
        };
        GameLoopState::Playing(witness)
    }
}

struct GameInstanceComponent;

fn drop_menu_witness(game: &game::Game, running: witness::Running) -> Witness {
    let choices = (0..game.inventory_size())
        .map(GameMenuChoice::DropItem)
        .collect::<Vec<_>>();
    let menu = GameMenu {
        text: "Select an item to drop (escape to cancel):".to_string(),
        choices,
    };
    running.menu(menu)
}

fn apply_menu_witness(game: &game::Game, running: witness::Running) -> Witness {
    let choices = (0..game.inventory_size())
        .map(GameMenuChoice::ApplyItem)
        .collect::<Vec<_>>();
    let menu = GameMenu {
        text: "Select an item to apply (escape to cancel):".to_string(),
        choices,
    };
    running.menu(menu)
}

pub enum GameLoopState {
    Paused(witness::Running),
    Playing(Witness),
    MainMenu,
    Help(witness::Running),
    MessageLog(witness::Running),
}

impl Component for GameInstanceComponent {
    type Output = GameLoopState;
    type State = GameLoopData;

    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        state.render(ctx, fb);
    }

    fn update(&mut self, state: &mut Self::State, _ctx: Ctx, event: Event) -> Self::Output {
        let running = witness::Running::cheat(); // XXX
        if event.is_escape() {
            GameLoopState::Paused(running)
        } else {
            state.update(event, running)
        }
    }

    fn size(&self, _state: &Self::State, ctx: Ctx) -> UCoord {
        ctx.bounding_box.size()
    }
}

fn menu_style<T: 'static>(menu: AppCF<T>) -> AppCF<T> {
    let mut border_style = BorderStyle::default();
    border_style.foreground = colours::TITLE_BACKGROUND.to_rgba32(255);
    menu.border(border_style)
        .fill(colours::MENU_BACKGROUND.to_rgba32(255))
        .centre()
        .overlay_tint(
            render_state(|state: &State, ctx, fb| state.render(ctx, fb)),
            chargrid::core::TintDim(63),
            60,
        )
}

#[derive(Clone)]
enum MainMenuEntry {
    NewGame,
    Help,
    Quit,
}

fn title_decorate<T: 'static>(cf: AppCF<T>) -> AppCF<T> {
    let decoration = {
        let style = Style::plain_text();
        chargrid::many![
            styled_string(
                " // // //    ROAD CLOSED    // // // ".to_string(),
                style
                    .with_bold(true)
                    .with_foreground(colours::TITLE_FOREGROUND.to_rgba32(255))
                    .with_background(colours::TITLE_BACKGROUND.to_rgba32(255)),
            )
            .add_offset(ICoord::new(21, 8)),
            styled_string(
                " // // // // // // // // // // // // ".to_string(),
                style
                    .with_bold(true)
                    .with_foreground(colours::TITLE_FOREGROUND.to_rgba32(255))
                    .with_background(colours::TITLE_BACKGROUND.to_rgba32(255)),
            )
            .add_offset(ICoord::new(21, 18))
        ]
    };
    cf.overlay(decoration, 0)
}

fn main_menu() -> AppCF<MainMenuEntry> {
    use MainMenuEntry::*;
    use menu::builder::*;
    let mut builder = menu_builder().vi_keys();
    let mut add_item = |entry, name, ch: char| {
        let identifier =
            MENU_FADE_SPEC.identifier(move |b| write!(b, "({}) {}", ch, name).unwrap());
        builder.add_item_mut(item(entry, identifier).add_hotkey_char(ch));
    };
    add_item(NewGame, "New Game", 'n');
    add_item(Help, "Help", 'h');
    if !cfg!(feature = "web") {
        add_item(Quit, "Quit", 'q');
    }
    builder.build_cf()
}

enum MainMenuOutput {
    NewGame { new_running: witness::Running },
    Quit,
}

const MAIN_MENU_TEXT_WIDTH: u32 = 40;

fn background() -> CF<(), State> {
    unit()
}

fn help() -> AppCF<()> {
    use chargrid::pad_by::Padding;
    menu_style(
        text::help(60)
            .pad_by(Padding {
                left: 1,
                right: 4,
                top: 1,
                bottom: 1,
            })
            .overlay(background(), 1),
    )
}

struct MessageLog {
    scroll_from_bottom: usize,
    reason: MessageLogReason,
}

impl Component for MessageLog {
    type Output = Option<()>;
    type State = GameLoopData;

    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        use chargrid::text::*;
        let ctx = ctx.set_size(self.size(state, ctx));
        match self.reason {
            MessageLogReason::Die => {
                Text::new(vec![StyledString {
                string: "Your final moments. Scroll with ↑↓. Press any other key to return to main menu.".to_string(),
                style: Style::plain_text().with_foreground(Rgba32::new_grey(187)),
            }])
            .wrap_word()
            .render(&(), ctx, fb);
            }
            MessageLogReason::Gameplay => {
                Text::new(vec![StyledString {
                    string: "Scroll with ↑↓. Press any other key to return to the game."
                        .to_string(),
                    style: Style::plain_text().with_foreground(Rgba32::new_grey(127)),
                }])
                .wrap_word()
                .render(&(), ctx, fb);
            }
            MessageLogReason::Win => {
                Text::new(vec![StyledString {
                    string: "You won. Scroll with ↑↓. Press any other key to return to main menu."
                        .to_string(),
                    style: Style::plain_text().with_foreground(Rgba32::new_grey(187)),
                }])
                .wrap_word()
                .render(&(), ctx, fb);
            }
        }
        let ctx = ctx.add_xy(0, 3);
        let instance = state.instance.as_ref().unwrap();
        let message_log = instance.game.inner_ref().message_log();
        let num_messages = message_log.len();
        if num_messages == 0 {
            StyledString {
                string: "(No messages in log.)".to_string(),
                style: Style::plain_text(),
            }
            .render(&(), ctx, fb);
        } else {
            let message_log_start = message_log
                .len()
                .saturating_sub(ctx.bounding_box.size().height() as usize)
                - self.scroll_from_bottom;
            for (i, message) in message_log[message_log_start..].iter().enumerate() {
                message_to_text(message.clone()).render(&(), ctx.add_y(i as i32), fb);
            }
        }
    }

    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        let ctx = ctx.set_size(self.size(state, ctx));
        let ctx = ctx.add_xy(0, 3);
        let instance = state.instance.as_ref().unwrap();
        let message_log = instance.game.inner_ref().message_log();
        let num_messages = message_log.len();
        if let Event::Input(Input::Keyboard(key)) = event {
            match key {
                KeyboardInput::Up => {
                    if num_messages > ctx.bounding_box.size().height() as usize {
                        let offset = num_messages - ctx.bounding_box.size().height() as usize;
                        let new_scroll = self.scroll_from_bottom + 1;
                        if new_scroll < offset {
                            self.scroll_from_bottom = new_scroll;
                        }
                    }
                }
                KeyboardInput::Down => {
                    self.scroll_from_bottom = self.scroll_from_bottom.saturating_sub(1);
                }
                _ => return Some(()),
            }
        }
        None
    }

    fn size(&self, _state: &Self::State, _ctx: Ctx) -> UCoord {
        UCoord::new(60, 25)
    }
}

#[derive(Debug, Clone, Copy)]
enum MessageLogReason {
    Gameplay,
    Die,
    Win,
}

fn message_log(reason: MessageLogReason) -> AppCF<()> {
    menu_style(cf(MessageLog {
        scroll_from_bottom: 0,
        reason,
    }))
}

fn main_menu_loop() -> AppCF<MainMenuOutput> {
    use MainMenuEntry::*;
    title_decorate(main_menu().add_offset(ICoord::new(34, 12))).repeat_unit(
        move |entry| match entry {
            NewGame => text::loading(MAIN_MENU_TEXT_WIDTH)
                .centre()
                .overlay(background(), 1)
                .then(|| {
                    on_state(|state: &mut State| MainMenuOutput::NewGame {
                        new_running: state.new_game(),
                    })
                })
                .break_(),
            Help => help().continue_(),
            Quit => val_once(MainMenuOutput::Quit).break_(),
        },
    )
}

#[derive(Clone)]
enum PauseMenuEntry {
    Resume,
    SaveQuit,
    Save,
    NewGame,
    Help,
    Clear,
}

fn pause_menu() -> AppCF<PauseMenuEntry> {
    use PauseMenuEntry::*;
    use menu::builder::*;
    let mut builder = menu_builder().vi_keys();
    let mut add_item = |entry, name, ch: char| {
        let identifier =
            MENU_FADE_SPEC.identifier(move |b| write!(b, "({}) {}", ch, name).unwrap());
        builder.add_item_mut(item(entry, identifier).add_hotkey_char(ch));
    };
    add_item(Resume, "Resume", 'r');
    if !cfg!(feature = "web") {
        add_item(SaveQuit, "Save and Quit", 'q');
        add_item(Save, "Save", 's');
    }
    add_item(NewGame, "New Game", 'n');
    add_item(Help, "Help", 'h');
    add_item(Clear, "Clear", 'c');
    builder.build_cf()
}

fn pause_menu_loop(running: witness::Running) -> AppCF<PauseOutput> {
    use PauseMenuEntry::*;
    let text_width = 64;
    pause_menu()
        .menu_harness()
        .repeat(
            running,
            move |running, entry_or_escape| match entry_or_escape {
                Ok(entry) => match entry {
                    Resume => break_(PauseOutput::ContinueGame { running }),
                    SaveQuit => text::saving(MAIN_MENU_TEXT_WIDTH)
                        .then(|| {
                            on_state(|state: &mut State| {
                                state.save_instance(running);
                                PauseOutput::Quit
                            })
                        })
                        .break_(),
                    Save => text::saving(MAIN_MENU_TEXT_WIDTH)
                        .then(|| {
                            on_state(|state: &mut State| PauseOutput::ContinueGame {
                                running: state.save_instance(running),
                            })
                        })
                        .break_(),
                    NewGame => text::loading(MAIN_MENU_TEXT_WIDTH)
                        .then(|| {
                            on_state(|state: &mut State| PauseOutput::ContinueGame {
                                running: state.new_game(),
                            })
                        })
                        .break_(),
                    Help => text::help(text_width).continue_with(running),
                    Clear => on_state(|state: &mut State| {
                        state.clear_saved_game();
                        PauseOutput::MainMenu
                    })
                    .break_(),
                },
                Err(_escape_or_start) => break_(PauseOutput::ContinueGame { running }),
            },
        )
}

enum PauseOutput {
    ContinueGame { running: witness::Running },
    MainMenu,
    Quit,
}

fn pause(running: witness::Running) -> AppCF<PauseOutput> {
    menu_style(pause_menu_loop(running))
}

fn game_instance_component(_running: witness::Running) -> AppCF<GameLoopState> {
    cf(GameInstanceComponent).some().no_peek()
}

fn win(win: game::Win) -> AppCF<()> {
    let text = match win {
        game::Win::Good => text::win(MAIN_MENU_TEXT_WIDTH),
        game::Win::Bad => text::bad_win(MAIN_MENU_TEXT_WIDTH),
    };
    menu_style(text)
        .then(|| message_log(MessageLogReason::Win))
        .map_side_effect(|_, state: &mut State| {
            state.clear_saved_game();
            state.save_config();
        })
}

fn game_over(reason: GameOverReason) -> AppCF<()> {
    menu_style(on_state_then(move |_state: &mut State| {
        text::game_over(MAIN_MENU_TEXT_WIDTH, reason)
    }))
    .then(|| message_log(MessageLogReason::Die))
    .map_side_effect(|_, state: &mut State| {
        state.clear_saved_game();
        state.save_config();
    })
}

fn apply_item_description(item: Item) -> String {
    use Item::*;
    match item {
        MedKit => "Apply to recover health.".to_string(),
        Firewood => "Apply at night to sleep for two hours and recover energy.".to_string(),
    }
}

fn menu_choice_string(game: &game::Game, choice: GameMenuChoice) -> String {
    match choice {
        GameMenuChoice::Empty => "(empty)".to_string(),
        GameMenuChoice::DropItem(i) => {
            if let Some(item) = game.inventory_item(i) {
                item_string_for_menu(item)
            } else {
                "(empty)".to_string()
            }
        }
        GameMenuChoice::ApplyItem(i) => {
            if let Some(item) = game.inventory_item(i) {
                format!(
                    "{} - {}",
                    item_string_for_menu(item),
                    apply_item_description(item)
                )
            } else {
                "(empty)".to_string()
            }
        }
    }
}

const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";

fn game_menu(menu_witness: witness::Menu) -> AppCF<Witness> {
    use game::MenuChoice;
    use menu::builder::*;
    let game_menu = menu_witness.menu.clone();
    let menu_cf = on_state_then(move |state: &mut State| {
        let instance = state.instance.as_ref().unwrap();
        let mut builder = menu_builder();
        let mut add_item = |entry: MenuChoice, name: String, ch: char| {
            let identifier =
                MENU_FADE_SPEC.identifier(move |b| write!(b, "{}) {}", ch, name).unwrap());
            builder.add_item_mut(item(entry, identifier).add_hotkey_char(ch));
        };
        for (choice, ch) in game_menu.choices.iter().zip(ALPHABET.chars()) {
            add_item(
                *choice,
                menu_choice_string(instance.game.inner_ref(), *choice),
                ch,
            );
        }
        let title = {
            use chargrid::text::*;
            Text::new(vec![StyledString {
                string: game_menu.text.clone(),
                style: Style::plain_text(),
            }])
            .wrap_word()
            .cf::<State>()
            .set_width(50)
        };
        let menu = builder
            .build_cf()
            .menu_harness()
            .with_title_vertical(title, 2);
        menu_style(menu)
    });
    menu_cf.and_then_side_effect(|result, state: &mut State| {
        let witness = match result {
            Err(Close) => menu_witness.cancel(),
            Ok(choice) => {
                if let Some(instance) = state.instance.as_mut() {
                    menu_witness.commit(&mut instance.game, choice)
                } else {
                    menu_witness.cancel()
                }
            }
        };
        val_once(witness)
    })
}

pub fn game_loop_component(initial_state: GameLoopState) -> AppCF<()> {
    use GameLoopState::*;
    loop_(initial_state, |state| match state {
        Playing(witness) => match witness {
            Witness::Running(running) => game_instance_component(running).continue_(),
            Witness::GameOver(reason) => game_over(reason).map_val(|| MainMenu).continue_(),
            Witness::Win(win_) => win(win_.win).map_val(|| MainMenu).continue_(),
            Witness::Menu(menu_) => game_menu(menu_).map(Playing).continue_(),
        },
        Paused(running) => pause(running).map(|pause_output| match pause_output {
            PauseOutput::ContinueGame { running } => {
                LoopControl::Continue(Playing(running.into_witness()))
            }
            PauseOutput::MainMenu => LoopControl::Continue(MainMenu),
            PauseOutput::Quit => LoopControl::Break(()),
        }),
        MainMenu => main_menu_loop().map(|main_menu_output| match main_menu_output {
            MainMenuOutput::NewGame { new_running } => {
                LoopControl::Continue(Playing(new_running.into_witness()))
            }
            MainMenuOutput::Quit => LoopControl::Break(()),
        }),
        Help(running) => help()
            .map(|()| GameLoopState::Playing(running.into_witness()))
            .continue_(),
        MessageLog(running) => message_log(MessageLogReason::Gameplay)
            .map(|()| GameLoopState::Playing(running.into_witness()))
            .continue_(),
    })
    .bound_size(UCoord::new_u16(80, 30))
    .on_exit_with_state(|state| state.try_save_instance_cheat())
}
