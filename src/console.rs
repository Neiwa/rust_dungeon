pub mod coord;

use crossterm::{event::KeyCode, style::Color};

use crate::{
    command::{AsCommand, Command},
    magic::Spell,
    AsCoord, AsDirection, Coord, Direction, Object, Player, Unit,
};

pub trait ConsoleUnit {
    fn color(&self) -> Color;
    fn symbol(&self) -> char;
    fn coord(&self) -> Coord;
}

impl ConsoleUnit for Player {
    fn color(&self) -> Color {
        Color::Cyan
    }

    fn symbol(&self) -> char {
        '🧙'
    }

    fn coord(&self) -> Coord {
        self.location.as_coord()
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

impl ConsoleUnit for Unit {
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
        SYMBOLS[self.id % SYMBOLS.len()]
    }

    fn coord(&self) -> Coord {
        self.location.as_coord()
    }
}

impl ConsoleUnit for dyn Object {
    fn color(&self) -> Color {
        self.get_spell().as_color()
    }

    fn symbol(&self) -> char {
        self.get_spell().as_symbol()
    }

    fn coord(&self) -> Coord {
        self.location().as_coord()
    }
}

pub trait AsColor {
    fn as_color(&self) -> Color;
}
pub trait AsSymbol {
    fn as_symbol(&self) -> char;
}

impl AsSymbol for Spell {
    fn as_symbol(&self) -> char {
        match self {
            Spell::Fireball => '🔥',
            Spell::Sphere => '🔵',
        }
    }
}
impl AsColor for Spell {
    fn as_color(&self) -> Color {
        match self {
            Spell::Fireball => Color::Red,
            Spell::Sphere => Color::Blue,
        }
    }
}

impl AsDirection for KeyCode {
    fn as_direction(self) -> Option<Direction> {
        match self {
            KeyCode::Up | KeyCode::Char('w') => Some(Direction::Up),
            KeyCode::Left | KeyCode::Char('a') => Some(Direction::Left),
            KeyCode::Down | KeyCode::Char('s') => Some(Direction::Down),
            KeyCode::Right | KeyCode::Char('d') => Some(Direction::Right),
            _ => None,
        }
    }
}

impl AsCommand for KeyCode {
    fn as_command(&self) -> Option<Command> {
        match self {
            KeyCode::Up | KeyCode::Char('w') => Some(Command::Move(Direction::Up)),
            KeyCode::Left | KeyCode::Char('a') => Some(Command::Move(Direction::Left)),
            KeyCode::Down | KeyCode::Char('s') => Some(Command::Move(Direction::Down)),
            KeyCode::Right | KeyCode::Char('d') => Some(Command::Move(Direction::Right)),
            KeyCode::Char('i') => Some(Command::Evoke(Direction::Up)),
            KeyCode::Char('j') => Some(Command::Evoke(Direction::Left)),
            KeyCode::Char('k') => Some(Command::Evoke(Direction::Down)),
            KeyCode::Char('l') => Some(Command::Evoke(Direction::Right)),
            KeyCode::Char('o') => Some(Command::CycleSpell),
            _ => None,
        }
    }
}

const LOADING_SYMBOLS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

pub fn loader(current: u128, target: u128, range: u128) -> char {
    let val = ((range.saturating_sub(target.saturating_sub(current))) as f32 / range as f32
        * LOADING_SYMBOLS.len() as f32)
        .clamp(0.0, (LOADING_SYMBOLS.len() - 1) as f32) as usize;
    LOADING_SYMBOLS[val]
}
