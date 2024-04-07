use crossterm::style::Color;

use crate::{monster::Monster, object::Object, player::Player};

use super::{AsColor, AsSymbol};

pub trait ConsoleUnit {
    fn color(&self) -> Color;
    fn symbol(&self) -> char;
}

impl ConsoleUnit for Player {
    fn color(&self) -> Color {
        Color::Cyan
    }

    fn symbol(&self) -> char {
        '🧙'
    }
}

const SYMBOLS: [char; 84] = [
    '🦇', '🪰', '🦟', '🐢', '🐈', '🐲', '🦍', '🦬', '🦌', '🦏', '🦛', '🐂', '🐃', '🐄', '🐖', '🐏',
    '🐑', '🐐', '🐪', '🐫', '🦙', '🦘', '🦥', '🦨', '🦡', '🐘', '🦣', '🐁', '🐀', '🦔', '🐇', '🦫',
    '🐉', '🦎', '🐊', '🐢', '🐍', '🦕', '🦖', '🦦', '🦈', '🐬', '🦭', '🐋', '🐟', '🐠', '🐡', '🦐',
    '🦑', '🐙', '🦞', '🦀', '🦆', '🐓', '🪼', '🦃', '🦅', '🦢', '🦜', '🪿', '🦩', '🦚', '🦉', '🦤',
    '🐦', '🐧', '🐥', '🐤', '🦋', '🐌', '🐛', '🪱', '🦗', '🐜', '🪳', '🐝', '🪲', '🐞', '🦂', '🦠',
    '🧞', '🧟', '🧌', '🫏',
];

impl ConsoleUnit for Monster {
    fn color(&self) -> Color {
        match self.id % 11 {
            0 => Color::Grey,
            1 => Color::DarkGrey,
            2 => Color::Red,
            3 => Color::DarkRed,
            4 => Color::Green,
            5 => Color::DarkGreen,
            6 => Color::Yellow,
            7 => Color::DarkYellow,
            8 => Color::Blue,
            9 => Color::DarkBlue,
            10 => Color::DarkMagenta,
            _ => Color::DarkCyan,
        }
    }

    fn symbol(&self) -> char {
        // let symbols = ['W', 'X', 'Y'];
        SYMBOLS[(self.id % SYMBOLS.len() as u64) as usize]
    }
}

impl ConsoleUnit for dyn Object {
    fn color(&self) -> Color {
        self.get_spell().as_color()
    }

    fn symbol(&self) -> char {
        self.get_spell().as_symbol()
    }
}

impl ConsoleUnit for &dyn Object {
    fn color(&self) -> Color {
        self.get_spell().as_color()
    }

    fn symbol(&self) -> char {
        self.get_spell().as_symbol()
    }
}
