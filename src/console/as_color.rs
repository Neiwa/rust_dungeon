use crossterm::style::Color;

use crate::magic::Spell;

pub trait AsColor {
    fn as_color(&self) -> Color;
}

impl AsColor for Spell {
    fn as_color(&self) -> Color {
        match self {
            Spell::Fireball => Color::Red,
            Spell::Sphere => Color::Blue,
            Spell::Inferno => Color::Red,
        }
    }
}
