use rgb_int::Rgb24;

pub const TITLE_BACKGROUND: Rgb24 = Rgb24::new(200, 150, 0);
pub const TITLE_FOREGROUND: Rgb24 = Rgb24::new_grey(0);
pub const MENU_FOREGROUND: Rgb24 = Rgb24::new_grey(187);
pub const MENU_BACKGROUND: Rgb24 = Rgb24::new_grey(0);
pub const MENU_SELECT_BACKGROUND: Rgb24 = TITLE_BACKGROUND.saturating_scalar_mul_div(3, 4);
pub const MENU_SELECT_FOREGROUND: Rgb24 = Rgb24::new_grey(255);
pub const BORDER: Rgb24 = TITLE_BACKGROUND;
pub const CURSOR: Rgb24 = TITLE_BACKGROUND;

pub const MED_KIT: Rgb24 = HEALTH.saturating_scalar_mul_div(2, 1);

pub const FLOOR: Rgb24 = Rgb24::new(108, 0, 162);
pub const VAPORWAVE_FOREGROUND: Rgb24 = Rgb24::new(255, 128, 255);
pub const VAPORWAVE_BACKGROUND: Rgb24 = Rgb24::new(68, 0, 102);
pub const ZOMBIE: Rgb24 = Rgb24::new(255, 51, 0);
pub const NORMAL_MODE: Rgb24 = Rgb24::new(0, 255, 255);
pub const HEALTH: Rgb24 = Rgb24::hex(0x800000);

pub const GROUND: Rgb24 = Rgb24::hex(0x326b0c);
pub const ROAD: Rgb24 = Rgb24::new_grey(187);
pub const TREE: Rgb24 = Rgb24::hex(0x0b4f17);
pub const CAR: Rgb24 = Rgb24::new(255, 32, 0);
