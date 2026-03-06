use rgb_int::Rgb24;

pub const TITLE_BACKGROUND: Rgb24 = Rgb24::new(200, 150, 0);
pub const TITLE_FOREGROUND: Rgb24 = Rgb24::new_grey(0);
pub const MENU_FOREGROUND: Rgb24 = Rgb24::new_grey(187);
pub const MENU_BACKGROUND: Rgb24 = Rgb24::new_grey(0);
pub const MENU_SELECT_BACKGROUND: Rgb24 = TITLE_BACKGROUND.saturating_scalar_mul_div(3, 4);
pub const MENU_SELECT_FOREGROUND: Rgb24 = Rgb24::new_grey(255);
pub const BORDER: Rgb24 = TITLE_BACKGROUND;
pub const CURSOR: Rgb24 = Rgb24::new_grey(63);

pub const MED_KIT: Rgb24 = HEALTH.saturating_scalar_mul_div(2, 1);
pub const FIREWOOD: Rgb24 = Rgb24::hex(0xeda12f);
pub const FRUIT: Rgb24 = Rgb24::hex(0xff14ac);
pub const FOOD_ITEM: Rgb24 = FOOD.saturating_scalar_mul_div(2, 1);
pub const COFFEE: Rgb24 = Rgb24::hex(0xC0FFEE);
pub const FUEL_CAN: Rgb24 = FUEL.saturating_scalar_mul_div(2, 1);

pub const ERROR: Rgb24 = Rgb24::new(200, 0, 0);

pub const ZOMBIE: Rgb24 = Rgb24::new(255, 51, 0);
pub const SLIME: Rgb24 = Rgb24::hex(0x0bedc3);
pub const NIGHT_STALKER: Rgb24 = Rgb24::hex(0x7744ed);
pub const CLIMBER: Rgb24 = Rgb24::hex(0xd9f909);
pub const DRAINER: Rgb24 = Rgb24::hex(0xf9096d);

pub const HEALTH: Rgb24 = Rgb24::hex(0x93150e);
pub const ENERGY: Rgb24 = Rgb24::hex(0x2c720e);
pub const FUEL: Rgb24 = Rgb24::hex(0x520b75);
pub const FOOD: Rgb24 = Rgb24::hex(0x5e4603);

pub const FLOOR: Rgb24 = Rgb24::new_grey(127);
pub const GROUND: Rgb24 = Rgb24::hex(0x326b0c);
pub const GRASS: Rgb24 = GROUND;
pub const ROAD: Rgb24 = Rgb24::new_grey(63);
pub const WALL: Rgb24 = Rgb24::new_grey(187);
pub const BRIDGE_RAILING: Rgb24 = Rgb24::new_grey(215);
pub const CLIFF_FACE: Rgb24 = Rgb24::new_grey(127);
pub const TREE: Rgb24 = Rgb24::hex(0x5db522);
pub const DEAD_TREE: Rgb24 = Rgb24::hex(0x875811);
pub const CABIN_WALL: Rgb24 = Rgb24::hex(0x96834b);
pub const CAR: Rgb24 = Rgb24::hex(0xdbd8d6);
pub const TYPEWRITER: Rgb24 = Rgb24::hex(0xe2c714);
pub const WEAPON: Rgb24 = Rgb24::hex(0xffffff);
pub const WATER: Rgb24 = Rgb24::hex(0x1a3ded);
