use crate::game_loop::{AppCF, State, Wish};
use chargrid::{
    control_flow::*,
    prelude::*,
    text::{StyledString, Text},
};
use game::GameOverReason;

fn text_component(width: u32, text: Vec<StyledString>) -> CF<(), State> {
    Text::new(text).wrap_word().cf().set_width(width)
}

pub fn help(width: u32) -> AppCF<()> {
    let t = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text(),
    };
    let b = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text().with_bold(true),
    };
    text_component(
        width,
        vec![
            b("Controls\n\n\n"),
            t("Walk: ←↑→↓\n\n"),
            t("Wait: Space\n\n"),
            t("Get item: g\n\n"),
            t("View inventory: i\n\n"),
            t("Apply item: a\n\n"),
            t("Drop item: d\n\n"),
            t("Unequip weapon: u\n\n"),
            t("Remove armour: r\n\n"),
            t("Display message log: m\n\n"),
            t("Display this help message: ?\n\n"),
        ],
    )
    .press_any_key()
    .then(move || help_1_5(width))
}

pub fn help_1_5(width: u32) -> AppCF<()> {
    let t = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text(),
    };
    let b = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text().with_bold(true),
    };
    text_component(
        width,
        vec![
            b("Tips\n\n\n"),
            t("- Light a fire (apply firewood) to sleep at night to regain energy and avoid night stalkers.\n\n"),
            t("- Get in the car by walking into one of its sides.\n\n"),
            t("- Access the car's inventory by walking into the back of it.\n\n"),
            t("- Walk into enemies to perform a melee attack.\n\n"),
            t("- There is no diagonal movement.\n\n"),
            t("- Mouse over a tile to see information abouti it, such as enemy health or an item's effect.\n\n"),
        ],
    )
    .press_any_key()
}

pub fn loading(width: u32) -> AppCF<()> {
    let t = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text(),
    };
    text_component(width, vec![t("Generating...")]).delay(Duration::from_millis(100))
}

pub fn saving(width: u32) -> AppCF<()> {
    let t = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text(),
    };
    text_component(width, vec![t("Saving...")]).delay(Duration::from_millis(100))
}

fn game_over_text(width: u32, _reason: GameOverReason) -> CF<(), State> {
    let t = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text(),
    };
    let text = vec![t("You have died.\n\nPress any key to continue...")];
    text_component(width, text)
}

pub fn game_over(width: u32, reason: GameOverReason) -> AppCF<()> {
    game_over_text(width, reason)
        .delay(Duration::from_secs(1))
        .then(move || game_over_text(width, reason).press_any_key())
}

fn win_text(width: u32) -> CF<(), State> {
    let t = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text(),
    };
    text_component(
        width,
        vec![t(
            "You defeated the CORRUPTOR and saved the city. Congratulations hero!",
        )],
    )
}
pub fn win(width: u32) -> AppCF<()> {
    // TODO: this is not ergonomic
    win_text(width)
        .delay(Duration::from_secs(1))
        .then(move || win_text(width).press_any_key())
}

fn bad_win_text(width: u32) -> CF<(), State> {
    let t = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text(),
    };
    text_component(
        width,
        vec![t(
            "With the CORRUPTED HEART beating in your chest you finally take your rightful place as the god of this world.",
        )],
    )
}
pub fn bad_win(width: u32) -> AppCF<()> {
    // TODO: this is not ergonomic
    bad_win_text(width)
        .delay(Duration::from_secs(1))
        .then(move || bad_win_text(width).press_any_key())
}

pub fn wishes(width: u32) -> AppCF<()> {
    let t = |s: String| StyledString {
        string: s,
        style: Style::plain_text(),
    };
    on_state_then(move |state: &mut State| {
        let wishes = &state.wishes;
        text_component(
            width,
            wishes
                .iter()
                .rev()
                .flat_map(|wish| {
                    vec![
                        StyledString {
                            string: format!("{}\n", wish.time),
                            style: Style::plain_text().with_foreground(Rgba32::new_grey(127)),
                        },
                        t(format!("{}\n\n", wish.wish)),
                    ]
                })
                .collect(),
        )
    })
    .press_any_key()
}
