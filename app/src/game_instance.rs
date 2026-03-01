use crate::colours;
use chargrid::{
    prelude::*,
    text::{self, StyledString, Text},
};
use game::{
    ActionError, CellVisibility, Config, Item, Layer, LayerTable, Message, Meter, Mode, NpcType,
    TerrainType, Tile, VisibleEntity,
    witness::{self, Game, RunningGame},
};
use rand::Rng;
use rgb_int::Rgb24;
use serde::{Deserialize, Serialize};

const GAME_DRAW_AREA_SIZE: UCoord = UCoord::new_u16(50, 25);
const CAR_ASCII_ART: &'static str = include_str!("car.txt");

#[derive(Clone, Copy)]
struct LightBlend {
    light_colour: Rgb24,
}

impl Tint for LightBlend {
    fn tint(&self, rgba32: Rgba32) -> Rgba32 {
        rgba32
            .to_rgb24()
            .normalised_mul(self.light_colour)
            .saturating_add(self.light_colour.saturating_scalar_mul_div(1, 5))
            .to_rgba32(255)
    }
}

pub struct GameInstance {
    pub game: Game,
}

enum DescriptionAt {
    Cursor,
    Player,
}

fn visible_entity_on_top(layers: &LayerTable<VisibleEntity>) -> Option<(&VisibleEntity, Layer)> {
    if layers.character.tile.is_some() {
        return Some((&layers.character, Layer::Character));
    }
    if layers.item.tile.is_some() {
        return Some((&layers.item, Layer::Item));
    }
    if layers.feature.tile.is_some() {
        return Some((&layers.feature, Layer::Feature));
    }
    if layers.floor.tile.is_some() {
        return Some((&layers.floor, Layer::Floor));
    }
    None
}

fn visible_entity_on_top_excluding_character(
    layers: &LayerTable<VisibleEntity>,
) -> Option<(&VisibleEntity, Layer)> {
    if layers.item.tile.is_some() {
        return Some((&layers.item, Layer::Item));
    }
    if layers.feature.tile.is_some() {
        return Some((&layers.feature, Layer::Feature));
    }
    if layers.floor.tile.is_some() {
        return Some((&layers.floor, Layer::Floor));
    }
    None
}

const METER_WIDTH: i32 = 11;

fn render_meter(meter: Meter, colour: Rgb24, ctx: Ctx, fb: &mut FrameBuffer) {
    use text::*;
    let width = METER_WIDTH as usize;
    let string = format!("{}/{}", meter.current(), meter.max());
    let style = Style::plain_text()
        .with_bold(true)
        .with_foreground(Rgb24::new_grey(255).to_rgba32(187));
    let centre_offset = (width / 2) - string.len().div_ceil(2) + 1;
    let filled_width = (meter.current() * width as u32) / meter.max().max(1);
    let filled_width = if filled_width == 0 && meter.current() > 0 {
        1
    } else {
        filled_width
    };
    for i in 0..width {
        let coord = ICoord::new(i as i32, 0);
        let alpha = if i < filled_width as usize { 255 } else { 63 };
        let rc = RenderCell::default().with_background(colour.to_rgba32(alpha));
        fb.set_cell_relative_to_ctx(ctx, coord, 0, rc);
    }
    StyledString { string, style }.render(&(), ctx.add_x(centre_offset as i32), fb);
}

impl GameInstance {
    pub fn new<R: Rng>(config: &Config, rng: &mut R) -> (Self, witness::Running) {
        let (game, running) = witness::new_game(config, rng);
        (GameInstance { game }, running)
    }

    pub fn into_storable(self, running: witness::Running) -> GameInstanceStorable {
        let Self { game } = self;
        let running_game = game.into_running_game(running);
        GameInstanceStorable { running_game }
    }

    fn layer_to_depth(layer: Layer) -> i8 {
        match layer {
            Layer::Character => 3,
            Layer::Item => 2,
            Layer::Feature => 1,
            Layer::Floor => 0,
        }
    }

    fn tile_to_render_cell(tile: Tile) -> RenderCell {
        match tile {
            Tile::Player => {
                return RenderCell {
                    character: Some('@'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(Rgba32::new_grey(255)),
                };
            }
            Tile::DeadPlayer => {
                return RenderCell {
                    character: Some('@'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(Rgba32::new(255, 0, 0, 255)),
                };
            }
            Tile::Ground => {
                return RenderCell {
                    character: Some('.'),
                    style: Style::new()
                        .with_bold(false)
                        .with_foreground(colours::GROUND.to_rgba32(255)),
                };
            }
            Tile::Road => {
                return RenderCell {
                    character: Some('.'),
                    style: Style::new()
                        .with_bold(false)
                        .with_foreground(colours::ROAD.to_rgba32(255)),
                };
            }
            Tile::Tree => {
                return RenderCell {
                    character: Some('&'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(colours::TREE.to_rgba32(255)),
                };
            }
            Tile::DeadTree => {
                return RenderCell {
                    character: Some('&'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(colours::DEAD_TREE.to_rgba32(255)),
                };
            }
            Tile::Wall => {
                return RenderCell {
                    character: Some('#'),
                    style: Style::new()
                        .with_bold(false)
                        .with_foreground(colours::WALL.to_rgba32(255)),
                };
            }
            Tile::Debris => {
                return RenderCell {
                    character: Some('%'),
                    style: Style::new()
                        .with_bold(false)
                        .with_foreground(Rgba32::new_grey(255)),
                };
            }
            Tile::DoorClosed => {
                return RenderCell {
                    character: Some('+'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(Rgba32::new_grey(255)),
                };
            }
            Tile::DoorOpen => {
                return RenderCell {
                    character: Some('-'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(Rgba32::new_grey(255)),
                };
            }
            Tile::Item(Item::MedKit) => {
                return RenderCell {
                    character: Some('{'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(colours::MED_KIT.to_rgba32(255)),
                };
            }
            Tile::Item(Item::Firewood) => {
                return RenderCell {
                    character: Some('*'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(colours::FIREWOOD.to_rgba32(255)),
                };
            }
            Tile::Zombie => {
                return RenderCell {
                    character: Some('z'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(colours::ZOMBIE.to_rgba32(255)),
                };
            }
            Tile::ZombieCorpse => {
                return RenderCell {
                    character: Some('?'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(colours::ZOMBIE.to_rgba32(255)),
                };
            }
            Tile::NightStalker => {
                return RenderCell {
                    character: Some('n'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(colours::NIGHT_STALKER.to_rgba32(255)),
                };
            }

            Tile::Car(ch) => {
                return RenderCell {
                    character: Some(ch),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(colours::CAR.to_rgba32(255)),
                };
            }
            Tile::FallenTree => {
                return RenderCell {
                    character: Some('%'),
                    style: Style::new()
                        .with_bold(false)
                        .with_foreground(colours::DEAD_TREE.to_rgba32(255)),
                };
            }
            Tile::Grass => {
                return RenderCell {
                    character: Some('"'),
                    style: Style::new()
                        .with_bold(false)
                        .with_foreground(colours::GRASS.to_rgba32(255)),
                };
            }
            Tile::CrushedGrass => {
                return RenderCell {
                    character: Some('\''),
                    style: Style::new()
                        .with_bold(false)
                        .with_foreground(colours::GRASS.to_rgba32(255)),
                };
            }
            Tile::CabinWall => {
                return RenderCell {
                    character: Some('#'),
                    style: Style::new()
                        .with_bold(false)
                        .with_foreground(colours::CABIN_WALL.to_rgba32(255)),
                };
            }
            Tile::Window => {
                return RenderCell {
                    character: Some('='),
                    style: Style::new()
                        .with_bold(false)
                        .with_foreground(colours::CABIN_WALL.to_rgba32(255)),
                };
            }
            Tile::Floor => {
                return RenderCell {
                    character: Some('.'),
                    style: Style::new()
                        .with_bold(false)
                        .with_foreground(colours::FLOOR.to_rgba32(255)),
                };
            }
        }
    }

    fn render_car(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        for (i, line) in CAR_ASCII_ART.lines().enumerate() {
            for (j, ch) in line.chars().enumerate() {
                let coord = ICoord::new(j as i32, i as i32);
                let foreground = match ch {
                    _ => colours::CAR.saturating_scalar_mul_div(2, 3),
                };
                fb.set_cell_relative_to_ctx(
                    ctx,
                    coord,
                    20,
                    RenderCell {
                        character: Some(ch),
                        style: Style::new()
                            .with_foreground(foreground.to_rgba32(255))
                            .with_bold(true),
                    },
                );
            }
        }
    }

    fn render_game(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        let game = self.game.inner_ref();
        let centre_coord_delta = game.player_coord() - GAME_DRAW_AREA_SIZE.to_icoord() / 2;
        for coord in GAME_DRAW_AREA_SIZE.icoord_iter_row_major() {
            let cell = game.cell_visibility_at_coord(coord + centre_coord_delta);
            let unseen_background = Rgba32::new(0, 0, 0, 255);
            match cell {
                CellVisibility::Never => {
                    let render_cell = RenderCell {
                        character: None,
                        style: Style::new().with_background(unseen_background),
                    };
                    fb.set_cell_relative_to_ctx(ctx, coord, 0, render_cell);
                }
                CellVisibility::Previous(data) => {
                    let background = Rgba32::new(0, 0, 0, 255);
                    data.tiles.for_each_enumerate(|visible_entity, layer| {
                        if let Some(&tile) = visible_entity.tile.as_ref() {
                            let depth = Self::layer_to_depth(layer);
                            let mut render_cell = Self::tile_to_render_cell(tile);
                            render_cell.style.background = Some(background);
                            render_cell.style.foreground = Some(Rgba32::new_grey(31));
                            fb.set_cell_relative_to_ctx(ctx, coord, depth, render_cell);
                        }
                    });
                }
                CellVisibility::Current { data, light_colour } => {
                    let light_colour = light_colour.unwrap_or(Rgb24::new_grey(0));
                    let tint = LightBlend { light_colour };
                    data.tiles.for_each_enumerate(|visible_entity, layer| {
                        if let Some(tile) = visible_entity.tile {
                            let depth = Self::layer_to_depth(layer);
                            let mut render_cell = Self::tile_to_render_cell(tile);
                            if let Some(colour_hint) = visible_entity.colour_hint {
                                render_cell = render_cell.with_foreground(colour_hint);
                            }
                            let tint = ctx.compose_tint(&tint);
                            fb.set_cell_relative_to_ctx(
                                ctx.with_tint(&tint),
                                coord,
                                depth,
                                render_cell,
                            );
                        }
                    });
                }
            }
        }
    }

    fn render_messages(&self, max_num_lines: u32, ctx: Ctx, fb: &mut FrameBuffer) {
        use text::*;
        let max = max_num_lines as usize;
        let mut messages: Vec<(usize, Message)> = Vec::new();
        for m in self.game.inner_ref().message_log().iter().rev() {
            if messages.len() >= max {
                break;
            }
            if let Some((count, last)) = messages.last_mut() {
                if last == m {
                    *count += 1;
                    continue;
                }
            }
            messages.push((1, m.clone()));
        }
        let mut line_count = 0;
        let mut offset = max_num_lines as i32;
        for (i, (count, m)) in messages.into_iter().enumerate() {
            if line_count >= max_num_lines {
                break;
            }
            let text = message_to_text(m);
            let mut text = if count == 1 {
                text
            } else {
                let mut text = text;
                text.parts
                    .push(StyledString::plain_text(format!(" (x{})", count)));
                text
            };
            text.parts = [vec![StyledString::plain_text("".to_string())], text.parts].concat();
            let alpha = 255 - (i * 30).min(220) as u8;
            for part in &mut text.parts {
                part.style = part.style.with_foreground(
                    part.style
                        .foreground
                        .unwrap_or_else(|| Rgba32::new(255, 255, 255, 255))
                        .with_a(alpha),
                );
            }
            let text = text.wrap_word();
            let size = text.size(&(), ctx);
            offset -= size.height() as i32;
            line_count += size.height();
            text.render(&(), ctx.add_y(offset), fb);
        }
    }

    fn render_description(
        &self,
        ctx: Ctx,
        fb: &mut FrameBuffer,
        cursor: Option<ICoord>,
    ) -> DescriptionAt {
        use text::*;
        let (cursor, player) = if let Some(cursor) = cursor {
            if cursor.is_valid(GAME_DRAW_AREA_SIZE) {
                let offset =
                    self.game.inner_ref().player_coord() - GAME_DRAW_AREA_SIZE.to_icoord() / 2;
                let cursor = cursor + offset;
                if self.game.inner_ref().world_size().is_valid(cursor) {
                    (cursor, false)
                } else {
                    (self.game.inner_ref().player_coord(), true)
                }
            } else {
                (self.game.inner_ref().player_coord(), true)
            }
        } else {
            (self.game.inner_ref().player_coord(), true)
        };
        let ret = if player {
            DescriptionAt::Player
        } else {
            DescriptionAt::Cursor
        };
        let (visible_entity, verb, end, currently_visible) =
            match self.game.inner_ref().cell_visibility_at_coord(cursor) {
                CellVisibility::Never => {
                    Text::new(vec![StyledString {
                        string: "UNDISCOVERED LOCATION".to_string(),
                        style: Style::new().with_foreground(Rgb24::new_grey(255).to_rgba32(127)),
                    }])
                    .wrap_word()
                    .render(&(), ctx, fb);
                    return ret;
                }
                CellVisibility::Previous(data) => (
                    visible_entity_on_top(&data.tiles),
                    "remember seeing",
                    Some("here"),
                    false,
                ),
                CellVisibility::Current { data, .. } => {
                    if player {
                        (
                            visible_entity_on_top_excluding_character(&data.tiles),
                            "see",
                            None,
                            true,
                        )
                    } else {
                        (visible_entity_on_top(&data.tiles), "see", None, true)
                    }
                }
            };
        if let Some((visible_entity, layer)) = visible_entity {
            if let Some(tile) = visible_entity.tile {
                if player {
                    let Description {
                        mut name,
                        description,
                    } = describe_tile(tile);
                    let mut text = Text {
                        parts: vec![StyledString::plain_text("Here: ".to_string())],
                    };
                    text.parts.append(&mut name.parts);
                    if let Some(end) = end {
                        text.parts.push(StyledString::plain_text(format!(" {end}")));
                    }
                    if let Some(mut description) = description {
                        text.parts
                            .push(StyledString::plain_text("\n\n".to_string()));
                        text.parts.append(&mut description.parts);
                    }
                    match layer {
                        Layer::Item => text.parts.push(StyledString {
                            string: "\n\n(Press g to pick it up.)".to_string(),
                            style: Style::new()
                                .with_foreground(Rgb24::new_grey(255).to_rgba32(127)),
                        }),
                        _ => text.parts.push(StyledString {
                            string: "\n\n(Move the cursor over a tile to see a description.)"
                                .to_string(),
                            style: Style::new()
                                .with_foreground(Rgb24::new_grey(255).to_rgba32(127)),
                        }),
                    }
                    text.wrap_word().render(&(), ctx, fb);
                } else {
                    let Description {
                        mut name,
                        description,
                    } = describe_tile(tile);
                    let mut text = Text {
                        parts: vec![StyledString::plain_text(format!("You {verb} "))],
                    };
                    text.parts.append(&mut name.parts);
                    if let Some(end) = end {
                        text.parts.push(StyledString::plain_text(format!(" {end}")));
                    }
                    text.parts.push(StyledString::plain_text(".".to_string()));
                    if currently_visible {
                        if let Some(health) = visible_entity.health {
                            if tile != Tile::Player {
                                text.parts
                                    .push(StyledString::plain_text("\n\n".to_string()));
                                text.parts
                                    .push(StyledString::plain_text("Its health is ".to_string()));
                                text.parts.push(StyledString {
                                    string: format!("{}/{}", health.current(), health.max()),
                                    style: Style::default().with_bold(true).with_foreground(
                                        colours::HEALTH
                                            .to_rgba32(255)
                                            .saturating_scalar_mul_div(3, 2),
                                    ),
                                });
                                text.parts.push(StyledString::plain_text(".".to_string()));
                            }
                        }
                    }
                    if let Some(mut description) = description {
                        text.parts
                            .push(StyledString::plain_text("\n\n".to_string()));
                        text.parts.append(&mut description.parts);
                    }
                    text.wrap_word().render(&(), ctx, fb);
                }
            }
        }
        ret
    }

    fn render_hint(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        use text::*;
        let text = Text::new(vec![
            StyledString::plain_text("You are the ".to_string()),
            StyledString {
                string: "@".to_string(),
                style: Style::plain_text().with_bold(true),
            },
            StyledString::plain_text(".\nMove with ←↑→↓.\nPress ? for more info.".to_string()),
        ]);
        text.wrap_word().render(&(), ctx, fb);
    }

    fn render_stats(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        use text::*;
        let x = 1;
        Text::new(vec![
            StyledString {
                string: "Time: ".to_string(),
                style: Style::plain_text(),
            },
            StyledString {
                string: format!("{}", self.game.inner_ref().time_of_day()),
                style: Style::plain_text().with_bold(true),
            },
        ])
        .render(&(), ctx.add_x(x), fb);

        let ctx = ctx.add_y(2);
        let stats = self.game.inner_ref().player_stats();
        let x = 1;
        StyledString {
            string: "Health:".to_string(),
            style: Style::plain_text(),
        }
        .render(&(), ctx.add_x(x), fb);
        let x = x + 8;
        render_meter(stats.health, colours::HEALTH, ctx.add_x(x), fb);
        let x = x + METER_WIDTH + 2;
        StyledString {
            string: "Energy:".to_string(),
            style: Style::plain_text(),
        }
        .render(&(), ctx.add_x(x), fb);
        let x = x + 8;
        render_meter(stats.energy, colours::ENERGY, ctx.add_x(x), fb);
        let x = x + METER_WIDTH + 2;
        StyledString {
            string: "Food:".to_string(),
            style: Style::plain_text(),
        }
        .render(&(), ctx.add_x(x), fb);
        let x = x + 6;
        render_meter(stats.food, colours::FOOD, ctx.add_x(x), fb);
        let x = x + METER_WIDTH + 2;
        StyledString {
            string: "Fuel:".to_string(),
            style: Style::plain_text(),
        }
        .render(&(), ctx.add_x(x), fb);
        let x = x + 6;
        render_meter(stats.car_fuel, colours::FUEL, ctx.add_x(x), fb);
    }

    pub fn render(&self, ctx: Ctx, fb: &mut FrameBuffer, cursor: Option<ICoord>, offset: ICoord) {
        use text::*;
        let border_style = Style::new()
            .with_bold(true)
            .with_foreground(colours::BORDER.to_rgba32(255));
        let border_text_style = Style::new()
            .with_bold(true)
            .with_foreground(colours::BORDER.to_rgba32(255));
        let game_size = GAME_DRAW_AREA_SIZE;
        self.render_stats(ctx.add_y(game_size.height() as i32 + 1), fb);
        match self.game.inner_ref().mode() {
            Mode::Driving => {
                self.render_car(ctx.add_xy(12, 1), fb);
                // line under game
                {
                    let box_render_cell = RenderCell::default().with_style(border_style);
                    let render_cell = box_render_cell.with_character('═');
                    for i in 0..fb.size().width() {
                        let coord = ICoord::new(i as i32, game_size.height() as i32);
                        fb.set_cell_relative_to_ctx(ctx, coord, 0, render_cell);
                    }
                }
                {
                    let ctx = ctx.add_xy(10, 15).set_width(fb.size().width() - 20);
                    let terrain_type = self.game.inner_ref().terrain_type();
                    terrain_type_text(terrain_type)
                        .wrap_word()
                        .render(&(), ctx, fb);
                }
                {
                    let ctx = ctx.add_xy(10, 22);
                    Text::new(vec![
                        StyledString {
                            string: "(d) continue driving\n".to_string(),
                            style: Style::plain_text(),
                        },
                        StyledString {
                            string: "(s) stop driving\n".to_string(),
                            style: Style::plain_text(),
                        },
                    ])
                    .render(&(), ctx, fb);
                }
            }
            Mode::Walking => {
                let box_render_cell = RenderCell::default().with_style(border_style);
                self.render_game(ctx.add_offset(offset), fb);
                // line under game
                {
                    for i in 0..fb.size().width() {
                        let render_cell = box_render_cell.with_character('═');
                        let coord = ICoord::new(i as i32, game_size.height() as i32);
                        fb.set_cell_relative_to_ctx(ctx, coord, 0, render_cell);
                    }
                    for i in 0..game_size.height() {
                        let render_cell = box_render_cell.with_character('║');
                        let coord = ICoord::new(game_size.width() as i32, i as i32);
                        fb.set_cell_relative_to_ctx(ctx, coord, 0, render_cell);
                    }
                    fb.set_cell_relative_to_ctx(
                        ctx,
                        game_size.to_icoord(),
                        0,
                        box_render_cell.with_character('╩'),
                    );
                }
                let messages_height = 9;
                let mut ui_y = 0;
                let ui_x = game_size.width() as i32 + 1;
                let ui_width = fb.size().width() - ui_x as u32;
                let ui_ctx = ctx.add_x(ui_x).set_width(ui_width + 1);
                for i in 0..ui_width {
                    let render_cell = box_render_cell.with_character('═');
                    let coord = ICoord::new(i as i32, ui_y);
                    fb.set_cell_relative_to_ctx(ui_ctx, coord, 1, render_cell);
                }
                Text::new(vec![
                    StyledString {
                        string: "╠══════╡ ".to_string(),
                        style: border_style,
                    },
                    StyledString {
                        string: "Message Log".to_string(),
                        style: border_text_style,
                    },
                    StyledString {
                        string: " ╞".to_string(),
                        style: border_style,
                    },
                ])
                .render(
                    &(),
                    ctx.add_xy(game_size.width() as i32, ui_y).add_depth(1),
                    fb,
                );
                self.render_messages(messages_height, ui_ctx.add_y(ui_y), fb);
                ui_y += messages_height as i32 + 1;
                for i in 0..ui_width {
                    let render_cell = box_render_cell.with_character('═');
                    let coord = ICoord::new(i as i32, ui_y);
                    fb.set_cell_relative_to_ctx(ui_ctx, coord, 1, render_cell);
                }
                let description_at = self.render_description(ui_ctx.add_y(ui_y + 2), fb, cursor);
                let description_at_text = match description_at {
                    DescriptionAt::Cursor => "at cursor",
                    DescriptionAt::Player => "at player",
                };
                Text::new(vec![
                    StyledString {
                        string: "╠═╡ ".to_string(),
                        style: border_style,
                    },
                    StyledString {
                        string: "Description".to_string(),
                        style: border_text_style,
                    },
                    StyledString {
                        string: format!(" ({})", description_at_text),
                        style: border_style.with_bold(false),
                    },
                    StyledString {
                        string: " ╞".to_string(),
                        style: border_style,
                    },
                ])
                .render(
                    &(),
                    ctx.add_xy(game_size.width() as i32, ui_y).add_depth(1),
                    fb,
                );
                ui_y += 10;
                for i in 0..ui_width {
                    let render_cell = box_render_cell.with_character('═');
                    let coord = ICoord::new(i as i32, ui_y);
                    fb.set_cell_relative_to_ctx(ui_ctx, coord, 1, render_cell);
                }
                Text::new(vec![
                    StyledString {
                        string: "╠══════════╡ ".to_string(),
                        style: border_style,
                    },
                    StyledString {
                        string: "Hint".to_string(),
                        style: border_text_style,
                    },
                    StyledString {
                        string: " ╞".to_string(),
                        style: border_style,
                    },
                ])
                .render(
                    &(),
                    ctx.add_xy(game_size.width() as i32, ui_y).add_depth(1),
                    fb,
                );
                self.render_hint(ui_ctx.add_y(ui_y + 1), fb);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameInstanceStorable {
    running_game: RunningGame,
}

impl GameInstanceStorable {
    pub fn into_game_instance(self) -> (GameInstance, witness::Running) {
        let Self { running_game } = self;
        let (game, running) = running_game.into_game();
        (GameInstance { game }, running)
    }
}

struct Description {
    name: Text,
    description: Option<Text>,
}

fn describe_tile(tile: Tile) -> Description {
    use text::*;
    match tile {
        Tile::Player => Description {
            name: Text::new(vec![StyledString::plain_text("yourself".to_string())]),
            description: None,
        },
        Tile::DeadPlayer => Description {
            name: Text::new(vec![StyledString::plain_text(
                "yourself (dead)".to_string(),
            )]),
            description: None,
        },
        Tile::Ground => Description {
            name: Text::new(vec![StyledString::plain_text("the ground".to_string())]),
            description: None,
        },
        Tile::Road => Description {
            name: Text::new(vec![StyledString::plain_text("the road".to_string())]),
            description: None,
        },
        Tile::Tree => Description {
            name: Text::new(vec![StyledString::plain_text("a tree".to_string())]),
            description: None,
        },
        Tile::Wall => Description {
            name: Text::new(vec![StyledString::plain_text("a wall".to_string())]),
            description: None,
        },
        Tile::DoorClosed => Description {
            name: Text::new(vec![StyledString::plain_text("a closed door".to_string())]),
            description: None,
        },
        Tile::DoorOpen => Description {
            name: Text::new(vec![StyledString::plain_text("an open door".to_string())]),
            description: None,
        },
        Tile::Debris => Description {
            name: Text::new(vec![StyledString::plain_text("some debris".to_string())]),
            description: None,
        },
        Tile::Item(Item::MedKit) => Description {
            name: Text::new(vec![
                StyledString::plain_text("a ".to_string()),
                StyledString {
                    string: "medkit".to_string(),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(colours::MED_KIT.to_rgba32(255)),
                },
            ]),
            description: Some(Text::new(vec![
                StyledString::plain_text("Apply to recover ".to_string()),
                StyledString {
                    string: "health.".to_string(),
                    style: Style::new().with_bold(true).with_foreground(
                        colours::HEALTH
                            .to_rgba32(255)
                            .saturating_scalar_mul_div(3, 2),
                    ),
                },
                StyledString::plain_text(".".to_string()),
            ])),
        },
        Tile::Item(Item::Firewood) => Description {
            name: Text::new(vec![
                StyledString::plain_text("a piece of ".to_string()),
                StyledString {
                    string: "firewood".to_string(),
                    style: Style::new().with_bold(true).with_foreground(
                        colours::FIREWOOD
                            .to_rgba32(255)
                            .saturating_scalar_mul_div(3, 2),
                    ),
                },
            ]),
            description: Some(Text::new(vec![
                StyledString::plain_text(
                    "Apply at night to sleep for two hours and recover ".to_string(),
                ),
                StyledString {
                    string: "energy".to_string(),
                    style: Style::new().with_bold(true).with_foreground(
                        colours::ENERGY
                            .to_rgba32(255)
                            .saturating_scalar_mul_div(3, 2),
                    ),
                },
                StyledString::plain_text(".".to_string()),
            ])),
        },
        Tile::Zombie => Description {
            name: Text::new(vec![
                StyledString::plain_text("a ".to_string()),
                StyledString {
                    string: "zombie".to_string(),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(colours::ZOMBIE.to_rgba32(255)),
                },
            ]),
            description: Some(Text::new(vec![StyledString::plain_text(
                "It moves slowly.\n\nIt doesn't stay dead.".to_string(),
            )])),
        },
        Tile::ZombieCorpse => Description {
            name: Text::new(vec![
                StyledString::plain_text("the corpse of a ".to_string()),
                StyledString {
                    string: "zombie".to_string(),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(colours::ZOMBIE.to_rgba32(255)),
                },
            ]),
            description: Some(Text::new(vec![StyledString::plain_text(
                "It will reanimate soon.".to_string(),
            )])),
        },
        Tile::NightStalker => Description {
            name: Text::new(vec![
                StyledString::plain_text("a ".to_string()),
                StyledString {
                    string: "night stalker".to_string(),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(colours::NIGHT_STALKER.to_rgba32(255)),
                },
            ]),
            description: Some(Text::new(vec![StyledString::plain_text(
                "It comes out at night.\n\nIt can climb.\n\nIt's afraid of fire.".to_string(),
            )])),
        },
        Tile::Car(_) => Description {
            name: Text::new(vec![StyledString::plain_text("your car".to_string())]),
            description: Some(Text::new(vec![StyledString::plain_text(
                "Walk into the car to start driving.".to_string(),
            )])),
        },
        Tile::DeadTree => Description {
            name: Text::new(vec![StyledString::plain_text("a dead tree".to_string())]),
            description: None,
        },
        Tile::FallenTree => Description {
            name: Text::new(vec![StyledString::plain_text("a fallen tree".to_string())]),
            description: None,
        },
        Tile::Grass => Description {
            name: Text::new(vec![StyledString::plain_text("some grass".to_string())]),
            description: None,
        },
        Tile::CrushedGrass => Description {
            name: Text::new(vec![StyledString::plain_text(
                "some crushed grass".to_string(),
            )]),
            description: None,
        },
        Tile::CabinWall => Description {
            name: Text::new(vec![StyledString::plain_text("a cabin wall".to_string())]),
            description: None,
        },
        Tile::Window => Description {
            name: Text::new(vec![StyledString::plain_text("a window".to_string())]),
            description: None,
        },
        Tile::Floor => Description {
            name: Text::new(vec![StyledString::plain_text("the floor".to_string())]),
            description: None,
        },
    }
}

fn terrain_type_text(terrain_type: TerrainType) -> Text {
    match terrain_type {
        TerrainType::PinePlantation => {
            Text::new(vec![
                StyledString::plain_text("You are driving along an abandoned fire trail through a pine plantation, overgrown with weeds. The trees, once organized into well-kempt rows, now stand at odd angles as they are reclaimed by nature.".to_string())
            ])
        }
    }
}

fn npc_type_to_styled_string(npc_type: NpcType) -> text::StyledString {
    use text::*;
    match npc_type {
        NpcType::Zombie => StyledString {
            string: "zombie".to_string(),
            style: Style::new()
                .with_bold(true)
                .with_foreground(colours::ZOMBIE.to_rgba32(255)),
        },
        NpcType::NightStalker => StyledString {
            string: "night stalker".to_string(),
            style: Style::new()
                .with_bold(true)
                .with_foreground(colours::NIGHT_STALKER.to_rgba32(255)),
        },
    }
}

pub fn message_to_text(message: Message) -> Text {
    use text::*;
    match message {
        Message::Wait => Text::new(vec![StyledString::plain_text(
            "You wait for a turn.".to_string(),
        )]),
        Message::OpenDoor => Text::new(vec![StyledString::plain_text(
            "You open the door.".to_string(),
        )]),
        Message::CloseDoor => Text::new(vec![StyledString::plain_text(
            "You close the door.".to_string(),
        )]),
        Message::ActionError(e) => Text::new(vec![StyledString::plain_text(match e {
            ActionError::InvalidMove => "You can't walk there.".to_string(),
            ActionError::MoveOutOfBounds => {
                "You don't want to walk too far from your car.".to_string()
            }
            ActionError::NothingToGet => "There is nothing here to pick up.".to_string(),
            ActionError::InventoryIsFull => {
                return Text::new(vec![
                    StyledString {
                        string: "Inv. is full. ".to_string(),
                        style: Style::plain_text(),
                    },
                    StyledString {
                        string: "(Press d to drop items.)".to_string(),
                        style: Style::plain_text()
                            .with_foreground(Rgb24::new_grey(127).to_rgba32(255)),
                    },
                ]);
            }
        })]),
        Message::NpcHit { npc_type, damage } => Text::new(vec![
            StyledString::plain_text("You hit the ".to_string()),
            npc_type_to_styled_string(npc_type),
            StyledString::plain_text(" for ".to_string()),
            StyledString {
                string: format!("{damage}"),
                style: Style::plain_text().with_bold(true),
            },
            StyledString::plain_text(" damage.".to_string()),
        ]),
        Message::NpcDies(npc_type) => Text::new(vec![
            StyledString::plain_text("The ".to_string()),
            npc_type_to_styled_string(npc_type),
            StyledString::plain_text(" dies.".to_string()),
        ]),
        Message::PlayerHit {
            attacker_npc_type,
            damage,
        } => Text::new(vec![
            StyledString::plain_text("The ".to_string()),
            npc_type_to_styled_string(attacker_npc_type),
            StyledString::plain_text(" hits you for ".to_string()),
            StyledString {
                string: format!("{damage}"),
                style: Style::plain_text().with_bold(true),
            },
            StyledString::plain_text(" damage.".to_string()),
        ]),
        Message::GetItem(item) => Text::new(vec![
            StyledString::plain_text("You pick up the ".to_string()),
            item_styled_string_for_message(item),
            StyledString::plain_text(".".to_string()),
        ]),
        Message::DropItem(item) => Text::new(vec![
            StyledString::plain_text("You drop the ".to_string()),
            item_styled_string_for_message(item),
            StyledString::plain_text(".".to_string()),
        ]),
        Message::YouDie => Text::new(vec![StyledString {
            string: "You die!".to_string(),
            style: Style::plain_text(),
        }]),
        Message::GetInCar => Text::new(vec![StyledString {
            string: "You get in the car.".to_string(),
            style: Style::plain_text(),
        }]),
        Message::GetOutOfCar => Text::new(vec![StyledString {
            string: "You get out of the car.".to_string(),
            style: Style::plain_text(),
        }]),
        Message::KickZombieCorpse => Text::new(vec![
            StyledString {
                string: "You kick the ".to_string(),
                style: Style::plain_text(),
            },
            StyledString {
                string: "zombie".to_string(),
                style: Style::plain_text()
                    .with_bold(true)
                    .with_foreground(colours::ZOMBIE.to_rgba32(255)),
            },
            StyledString {
                string: " corpse!".to_string(),
                style: Style::plain_text(),
            },
        ]),
        Message::DestroyZombieCorpse => Text::new(vec![
            StyledString {
                string: "The ".to_string(),
                style: Style::plain_text(),
            },
            StyledString {
                string: "zombie".to_string(),
                style: Style::plain_text()
                    .with_bold(true)
                    .with_foreground(colours::ZOMBIE.to_rgba32(255)),
            },
            StyledString {
                string: " corpse is destroyed!".to_string(),
                style: Style::plain_text(),
            },
        ]),
        Message::NightStalkerSpawn => Text::new(vec![StyledString {
            string: "You hear the nearby sound of claws scratching at the earth.".to_string(),
            style: Style::plain_text(),
        }]),
        Message::NightStalkerDespawn => Text::new(vec![
            StyledString {
                string: "The ".to_string(),
                style: Style::plain_text(),
            },
            StyledString {
                string: "night stalker".to_string(),
                style: Style::plain_text()
                    .with_bold(true)
                    .with_foreground(colours::NIGHT_STALKER.to_rgba32(255)),
            },
            StyledString {
                string: " burrows into the ground.".to_string(),
                style: Style::plain_text(),
            },
        ]),
    }
}

fn item_styled_string_for_message(item: Item) -> text::StyledString {
    use text::*;
    match item {
        Item::MedKit => StyledString {
            string: "medkit".to_string(),
            style: Style::new()
                .with_bold(true)
                .with_foreground(colours::MED_KIT.to_rgba32(255)),
        },
        Item::Firewood => StyledString {
            string: "firewood".to_string(),
            style: Style::new()
                .with_bold(true)
                .with_foreground(colours::FIREWOOD.to_rgba32(255)),
        },
    }
}

pub fn item_string_for_menu(item: Item) -> String {
    match item {
        Item::MedKit => "Medkit".to_string(),
        Item::Firewood => "Firewood".to_string(),
    }
}
