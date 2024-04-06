mod as_color;
mod as_symbol;
mod command;
mod console_unit;
mod coord;
mod direction;
mod display;
mod input;
mod input_tracker;

pub use self::as_color::AsColor;
pub use self::as_symbol::AsSymbol;
pub use self::console_unit::ConsoleUnit;
pub use self::coord::{AsCoord, Coord};
pub use self::direction::Direction;
pub use self::display::Display;
pub use self::input_tracker::InputTracker;

const LOADING_SYMBOLS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

#[allow(dead_code)]
pub fn loader(current: u128, target: u128, range: u128) -> char {
    let val = ((range.saturating_sub(target.saturating_sub(current))) as f32 / range as f32
        * LOADING_SYMBOLS.len() as f32)
        .clamp(0.0, (LOADING_SYMBOLS.len() - 1) as f32) as usize;
    LOADING_SYMBOLS[val]
}
pub fn loader_reverse(current: u128, target: u128, range: u128) -> char {
    let val = ((range.saturating_sub(target.saturating_sub(current))) as f32 / range as f32
        * LOADING_SYMBOLS.len() as f32)
        .clamp(0.0, (LOADING_SYMBOLS.len() - 1) as f32) as usize;
    LOADING_SYMBOLS[LOADING_SYMBOLS.len() - val - 1]
}
