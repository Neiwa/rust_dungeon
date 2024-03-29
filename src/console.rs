pub mod coord;

use crossterm::{event::KeyCode, style::Color};

use crate::{command::{AsCommand, Command}, AsCoord, AsDirection, Coord, Direction, Player, Unit};

pub trait ConsoleUnit {
    fn color(&self) -> Color;
    fn symbol(&self) -> char;
    fn coord(&self) -> Coord;
    fn last_coord(&self) -> Coord;
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

    fn last_coord(&self) -> Coord {
        self.last_coord
    }
}

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
        let symbols = [
            '🦇', '🪰', '🦟', '🐢', '🐈', '🐲', '🦍', '🦬', '🦌', '🦏', '🦛', '🐂', '🐃', '🐄',
            '🐖', '🐏', '🐑', '🐐', '🐪', '🐫', '🦙', '🦘', '🦥', '🦨', '🦡', '🐘', '🦣', '🐁',
            '🐀', '🦔', '🐇', '🦫', '🐉', '🦎', '🐊', '🐢', '🐍', '🦕', '🦖', '🦦', '🦈', '🐬',
            '🦭', '🐋', '🐟', '🐠', '🐡', '🦐', '🦑', '🐙', '🦞', '🦀', '🦆', '🐓', '🪼', '🦃',
            '🦅', '🦢', '🦜', '🪿', '🦩', '🦚', '🦉', '🦤', '🐦', '🐧', '🐥', '🐤', '🦋', '🐌',
            '🐛', '🪱', '🦗', '🐜', '🪳', '🐝', '🪲', '🐞', '🦂', '🦠', '🧞', '🧟', '🧌', '🫏',
        ];
        symbols[self.id % symbols.len()]
    }

    fn coord(&self) -> Coord {
        self.location.as_coord()
    }

    fn last_coord(&self) -> Coord {
        self.last_coord
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
            _ => None,
        }
    }
}
