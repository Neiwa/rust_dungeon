use crate::magic::Spell;

pub trait AsSymbol {
    fn as_symbol(&self) -> char;
}

impl AsSymbol for Spell {
    fn as_symbol(&self) -> char {
        match self {
            Spell::Fireball => '🔥',
            Spell::Sphere => '🔵',
            Spell::Inferno => '🎆',
        }
    }
}
