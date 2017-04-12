//pictures
pub static TRUMP: &'static [u8] = include_bytes!("../pics/trump_cartoon.dump");
pub static SUPER_TRUMP: &'static [u8] = include_bytes!("../pics/mexican_trump_head.dump");
pub static MEXICAN: &'static [u8] = include_bytes!("../pics/mexican_cartoon.dump");
pub static SUPER_MEXICAN: &'static [u8] = include_bytes!("../pics/super_mexican.dump");
pub static BACKGROUND: &'static [u8] = include_bytes!("../pics/desert.dump");
pub static START: &'static [u8] = include_bytes!("../pics/start.dump");
pub static GAMEOVER: &'static [u8] = include_bytes!("../pics/gameover.dump");
pub static TACO_MODE: &'static [u8] = include_bytes!("../pics/taco.dump");
pub static BURGER_MODE: &'static [u8] = include_bytes!("../pics/burger.dump");
pub static SILENT_BTN_NEG: &'static [u8] = include_bytes!("../pics/mic_on.dump");
pub static SILENT_BTN: &'static [u8] = include_bytes!("../pics/mic_off.dump");

// renderer
pub const DISPLAY_SIZE: (u16, u16) = (480, 272);
pub const GAME_OVER_SIZE: (u16, u16) = (480, 64);
pub const GAME_OVER_OFFSET_Y: u16 = 90;
pub const START_SIZE: (u16, u16) = (480, 74);
pub const SILENT_BTN_SIZE: (u16, u16) = (40, 40);
pub const GAME_MODE_BTN_SIZE: (u16, u16) = (80, 80);

// targets
pub const TARGET_SIZE_50: (u16, u16) = (50, 50);

// seven_segment
pub const ELEMENT_WIDTH_SMALL: u16 = 12;
pub const ELEMENT_GAP_SMALL: u16 = 3;
pub const ELEMENT_WIDTH_BIG: u16 = 24;
pub const ELEMENT_GAP_BIG: u16 = ELEMENT_GAP_SMALL;

// colors
pub const RED: u16 = 0xFC00;
pub const GREEN: u16 = 0x83E0;
pub const BLACK: u16 = 0x8000;

// game constants
pub const GAME_TIME: u16 = 30;
pub const MAX_EVIL_TARGETS: usize = 4;
pub const MAX_HERO_TARGETS: usize = 6;
pub const HERO_POINTS: u16 = 70;
pub const EVIL_POINTS: u16 = 50;
pub const SUPER_EVIL_POINTS: u16 = 100;
pub const SUPER_TARGET_HIDING_DURATION: (usize, usize) = (5000, 10000);
pub const VOLUME_THRESHOLD: u16 = 2000;
