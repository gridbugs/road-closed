use crate::colours;
use chargrid::{
    prelude::*,
    text::{self, Text},
};
use game::{
    witness::{self, Game, RunningGame},
    ActionError, CellVisibility, Config, Item, Layer, LayerTable, Message, Meter, NpcType, Tile,
    VisibleEntity,
};
use rand::Rng;
use rgb_int::Rgb24;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Normal,
}

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

fn render_meter(meter: Meter, colour: Rgb24, ctx: Ctx, fb: &mut FrameBuffer) {
    use text::*;
    let width = 15;
    let string = format!("{}/{}", meter.current(), meter.max());
    let style = Style::plain_text()
        .with_bold(true)
        .with_foreground(Rgb24::new_grey(255).to_rgba32(187));
    let centre_offset = (width / 2) - string.len().div_ceil(2);
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
            Tile::Floor => {
                return RenderCell {
                    character: Some('.'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(colours::FLOOR.to_rgba32(255)),
                };
            }
            Tile::Wall => {
                return RenderCell {
                    character: Some('#'),
                    style: Style::new()
                        .with_bold(false)
                        .with_foreground(colours::VAPORWAVE_FOREGROUND.to_rgba32(255)),
                };
            }
            Tile::Debris => {
                return RenderCell {
                    character: Some('%'),
                    style: Style::new()
                        .with_bold(true)
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
            Tile::Zombie => {
                return RenderCell {
                    character: Some('z'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(colours::ZOMBIE.to_rgba32(255)),
                };
            }
        }
    }

    pub fn render_game(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        let centre_coord_delta = ICoord::new(0, 0);
        for coord in ctx.bounding_box.size().icoord_iter_row_major() {
            let cell = self
                .game
                .inner_ref()
                .cell_visibility_at_coord(coord + centre_coord_delta);
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
                            render_cell.style.foreground = Some(Rgba32::new_grey(127));
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

    fn render_messages(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        use text::*;
        let max = 4;
        let mut messages: Vec<(usize, Message)> = Vec::new();
        for m in self.game.inner_ref().message_log().iter().rev() {
            if messages.len() >= max {
                break;
            }
            if let Some((ref mut count, last)) = messages.last_mut() {
                if last == m {
                    *count += 1;
                    continue;
                }
            }
            messages.push((1, m.clone()));
        }
        for (i, (count, m)) in messages.into_iter().enumerate() {
            let text = message_to_text(m);
            let mut text = if count == 1 {
                text
            } else {
                let mut text = text;
                text.parts
                    .push(StyledString::plain_text(format!(" (x{})", count)));
                text
            };
            let alpha = 255 - (i as u8 * 50);
            let offset = max as i32 - i as i32 - 1;
            for part in &mut text.parts {
                part.style = part.style.with_foreground(
                    part.style
                        .foreground
                        .unwrap_or_else(|| Rgba32::new(255, 255, 255, 255))
                        .with_a(alpha),
                );
            }
            text.render(&(), ctx.add_y(offset), fb);
        }
    }

    fn render_description(&self, ctx: Ctx, fb: &mut FrameBuffer, cursor: Option<ICoord>) {
        use text::*;
        let (cursor, player) = if let Some(cursor) = cursor {
            if self.game.inner_ref().world_size().is_valid(cursor) {
                (cursor, false)
            } else {
                (self.game.inner_ref().player_coord(), true)
            }
        } else {
            (self.game.inner_ref().player_coord(), true)
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
                    return;
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
                        parts: vec![StyledString::plain_text("There is ".to_string())],
                    };
                    text.parts.append(&mut name.parts);
                    if let Some(end) = end {
                        text.parts.push(StyledString::plain_text(format!(" {end}")));
                    }
                    text.parts
                        .push(StyledString::plain_text(" here.".to_string()));
                    if let Some(mut description) = description {
                        text.parts
                            .push(StyledString::plain_text("\n\n".to_string()));
                        text.parts.append(&mut description.parts);
                    }
                    match layer {
                        Layer::Floor => text.parts.push(StyledString {
                            string: "\n\n(Move the cursor over a tile to see a description.)"
                                .to_string(),
                            style: Style::new()
                                .with_foreground(Rgb24::new_grey(255).to_rgba32(127)),
                        }),
                        Layer::Item => text.parts.push(StyledString {
                            string: "\n\n(Press g to pick it up.)".to_string(),
                            style: Style::new()
                                .with_foreground(Rgb24::new_grey(255).to_rgba32(127)),
                        }),
                        _ => (),
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
    }

    fn render_mode(&self, ctx: Ctx, fb: &mut FrameBuffer, mode: Mode) {
        use text::*;
        let text = match mode {
            Mode::Normal => Text::new(vec![StyledString::plain_text(
                "Move with ←↑→↓.\nPress ? for more info.".to_string(),
            )]),
        };
        text.wrap_word().render(&(), ctx, fb);
    }

    fn render_stats(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        use text::*;
        let stats = self.game.inner_ref().player_stats();
        let x_offset = 11;
        StyledString {
            string: "Health:".to_string(),
            style: Style::plain_text(),
        }
        .render(&(), ctx, fb);
        render_meter(stats.health, colours::HEALTH, ctx.add_x(x_offset), fb);
    }

    pub fn render(
        &self,
        ctx: Ctx,
        fb: &mut FrameBuffer,
        cursor: Option<ICoord>,
        mode: Mode,
        offset: ICoord,
    ) {
        use text::*;
        self.render_game(ctx.add_offset(offset), fb);
        self.render_messages(
            ctx.add_xy(1, ctx.bounding_box.size().height() as i32 - 4)
                .add_depth(20),
            fb,
        );
        let border_style = Style::new()
            .with_bold(true)
            .with_foreground(colours::VAPORWAVE_BACKGROUND.to_rgba32(255));
        let border_text_style = Style::new()
            .with_bold(true)
            .with_foreground(colours::VAPORWAVE_FOREGROUND.to_rgba32(255));
        let game_size = self.game.inner_ref().world_size();
        let box_render_cell = RenderCell::default().with_style(border_style);
        // line to the right of game
        {
            let render_cell = box_render_cell.with_character('║');
            for i in 0..ctx.bounding_box.size().height() {
                let coord = ICoord::new(game_size.width() as i32, i as i32);
                fb.set_cell_relative_to_ctx(ctx, coord, 0, render_cell);
            }
        }
        // line under game
        {
            let render_cell = box_render_cell.with_character('═');
            for i in 0..game_size.width() {
                let coord = ICoord::new(i as i32, game_size.height() as i32);
                fb.set_cell_relative_to_ctx(ctx, coord, 0, render_cell);
            }
            Text::new(vec![
                StyledString {
                    string: "╡".to_string(),
                    style: border_style,
                },
                StyledString {
                    string: "Message Log".to_string(),
                    style: border_text_style,
                },
                StyledString {
                    string: " (press m to display full log)".to_string(),
                    style: Style::plain_text().with_foreground(Rgb24::new_grey(127).to_rgba32(255)),
                },
                StyledString {
                    string: "╞".to_string(),
                    style: border_style,
                },
            ])
            .render(&(), ctx.add_xy(2, game_size.height() as i32), fb);
        }
        fb.set_cell_relative_to_ctx(
            ctx,
            game_size.to_icoord(),
            0,
            box_render_cell.with_character('╣'),
        );
        // description
        {
            let offset_y = 21;
            let render_cell = box_render_cell.with_character('═');
            for i in (game_size.width() + 1)..ctx.bounding_box.size().width() {
                let coord = ICoord::new(i as i32, offset_y);
                fb.set_cell_relative_to_ctx(ctx, coord, 0, render_cell);
            }
            Text::new(vec![
                StyledString {
                    string: "╡".to_string(),
                    style: border_style,
                },
                StyledString {
                    string: "Description: ".to_string(),
                    style: border_text_style,
                },
                if cursor.is_some() {
                    match mode {
                        Mode::Normal => StyledString {
                            string: "AT CURSOR".to_string(),
                            style: border_text_style
                                .with_foreground(colours::NORMAL_MODE.to_rgba32(255)),
                        },
                    }
                } else {
                    StyledString {
                        string: "AT PLAYER".to_string(),
                        style: border_text_style.with_foreground(Rgba32::new_grey(255)),
                    }
                },
                StyledString {
                    string: "╞".to_string(),
                    style: border_style,
                },
            ])
            .render(&(), ctx.add_xy(game_size.width() as i32 + 1, offset_y), fb);
            fb.set_cell_relative_to_ctx(
                ctx,
                game_size.to_icoord().set_y(offset_y),
                0,
                box_render_cell.with_character('╠'),
            );
            self.render_description(
                ctx.add_offset(game_size.to_icoord().set_y(offset_y + 1))
                    .add_xy(2, 1),
                fb,
                cursor,
            );
        }
        // mode
        {
            let offset_y = 16;
            let render_cell = box_render_cell.with_character('═');
            for i in (game_size.width() + 1)..ctx.bounding_box.size().width() {
                let coord = ICoord::new(i as i32, offset_y);
                fb.set_cell_relative_to_ctx(ctx, coord, 0, render_cell);
            }
            Text::new(vec![
                StyledString {
                    string: "╡".to_string(),
                    style: border_style,
                },
                StyledString {
                    string: "Mode: ".to_string(),
                    style: border_text_style,
                },
                match mode {
                    Mode::Normal => StyledString {
                        string: "NORMAL".to_string(),
                        style: border_text_style
                            .with_foreground(colours::NORMAL_MODE.to_rgba32(255)),
                    },
                },
                StyledString {
                    string: "╞".to_string(),
                    style: border_style,
                },
            ])
            .render(&(), ctx.add_xy(game_size.width() as i32 + 1, offset_y), fb);
            fb.set_cell_relative_to_ctx(
                ctx,
                game_size.to_icoord().set_y(offset_y),
                0,
                box_render_cell.with_character('╠'),
            );
            self.render_mode(
                ctx.add_offset(game_size.to_icoord().set_y(offset_y + 1))
                    .add_xy(2, 1),
                fb,
                mode,
            );
        }
        // stats
        {
            let offset_y = 6;
            let render_cell = box_render_cell.with_character('═');
            for i in (game_size.width() + 1)..ctx.bounding_box.size().width() {
                let coord = ICoord::new(i as i32, offset_y);
                fb.set_cell_relative_to_ctx(ctx, coord, 0, render_cell);
            }
            Text::new(vec![
                StyledString {
                    string: "╡".to_string(),
                    style: border_style,
                },
                StyledString {
                    string: "Stats".to_string(),
                    style: border_text_style,
                },
                StyledString {
                    string: "╞".to_string(),
                    style: border_style,
                },
            ])
            .render(&(), ctx.add_xy(game_size.width() as i32 + 1, offset_y), fb);
            fb.set_cell_relative_to_ctx(
                ctx,
                game_size.to_icoord().set_y(offset_y),
                0,
                box_render_cell.with_character('╠'),
            );
            self.render_stats(
                ctx.add_offset(game_size.to_icoord().set_y(offset_y + 1))
                    .add_xy(2, 1),
                fb,
            );
        }
    }
}

#[derive(Serialize, Deserialize)]
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
        Tile::Floor => Description {
            name: Text::new(vec![StyledString::plain_text("the floor".to_string())]),
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
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(colours::HEALTH.to_rgba32(255)),
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
                "Doesn't stay dead.".to_string(),
            )])),
        },
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
            StyledString::plain_text("The ".to_string()),
            npc_type_to_styled_string(npc_type),
            StyledString::plain_text(" is hit for ".to_string()),
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
            style: Style::plain_text().with_foreground(Rgb24::new(255, 0, 0).to_rgba32(255)),
        }]),
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
    }
}

pub fn item_string_for_menu(item: Item) -> String {
    match item {
        Item::MedKit => "MedKit".to_string(),
    }
}
