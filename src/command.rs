use crate::direction::Direction;

pub enum Command {
    Move(Direction),
    Evoke(Direction),
    EvokeMouse,
    CycleSpell(bool),
    SelectSpell(usize),
}

pub trait AsCommand {
    fn as_command(&self) -> Option<Command>;
}
