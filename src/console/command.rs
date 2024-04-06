use crossterm::event::KeyCode;

use crate::{
    command::{AsCommand, Command},
    direction::Direction,
};

use super::input::{Input, InputState};

impl AsCommand for InputState {
    fn as_command(&self) -> Option<Command> {
        match self {
            InputState::Press(Input::Key(code)) => match code {
                KeyCode::Char('u') | KeyCode::Char('q') => Some(Command::CycleSpell(false)),
                KeyCode::Char('o') | KeyCode::Char('e') => Some(Command::CycleSpell(true)),
                KeyCode::Char('1') => Some(Command::SelectSpell(0)),
                KeyCode::Char('2') => Some(Command::SelectSpell(1)),
                KeyCode::Char('3') => Some(Command::SelectSpell(2)),
                KeyCode::Char('4') => Some(Command::SelectSpell(3)),
                KeyCode::Char('5') => Some(Command::SelectSpell(4)),
                KeyCode::Char('6') => Some(Command::SelectSpell(5)),
                KeyCode::Char('7') => Some(Command::SelectSpell(6)),
                KeyCode::Char('8') => Some(Command::SelectSpell(7)),
                KeyCode::Char('9') => Some(Command::SelectSpell(8)),
                _ => None,
            },
            InputState::Release(code) => match code {
                _ => None,
            },
            InputState::Active(Input::Key(code)) => match code {
                KeyCode::Up | KeyCode::Char('w') => Some(Command::Move(Direction::Up)),
                KeyCode::Left | KeyCode::Char('a') => Some(Command::Move(Direction::Left)),
                KeyCode::Down | KeyCode::Char('s') => Some(Command::Move(Direction::Down)),
                KeyCode::Right | KeyCode::Char('d') => Some(Command::Move(Direction::Right)),
                KeyCode::Char('i') => Some(Command::Evoke(Direction::Up)),
                KeyCode::Char('j') => Some(Command::Evoke(Direction::Left)),
                KeyCode::Char('k') => Some(Command::Evoke(Direction::Down)),
                KeyCode::Char('l') => Some(Command::Evoke(Direction::Right)),
                _ => None,
            },
            InputState::Active(Input::MouseLeft) => Some(Command::EvokeMouse),
            _ => None,
        }
    }
}
