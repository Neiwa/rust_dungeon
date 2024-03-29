use crate::Direction;

pub enum Command {
    Move(Direction),
    Fireball(Direction),
}

pub trait AsCommand {
    fn as_command(&self) -> Option<Command>;
}