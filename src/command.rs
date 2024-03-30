use crate::Direction;

pub enum Command {
    Move(Direction),
    Evoke(Direction),
    CycleSpell(bool),
    SelectSpell(usize),
}

pub trait AsCommand {
    fn as_command(&self) -> Option<Command>;
}
