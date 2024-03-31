use crate::{
    magic::{fireball::FireballMagic, inferno::InfernoMagic, sphere::SphereMagic, Magic},
    object::Object,
    point::Point,
    Coord, Direction, Entity, Unit,
};

pub struct Player {
    pub location: Point,
    pub energy: u32,
    pub max_energy: u32,
    pub spells: Vec<Box<dyn Magic>>,
    pub active_spell: usize,
}

impl Player {
    pub fn new(coord: Coord) -> Self {
        Self {
            location: Point::new(coord.x as f64, coord.y as f64),
            energy: 100,
            max_energy: 100,
            spells: vec![
                Box::new(FireballMagic::new()),
                Box::new(SphereMagic::new()),
                Box::new(InfernoMagic::new()),
                Box::new(InfernoMagic::new()),
            ],
            active_spell: 1,
        }
    }

    pub fn get_active_spell(&self) -> &Box<dyn Magic> {
        self.spells.get(self.active_spell).unwrap()
    }

    pub fn active_spell_evoke(
        &mut self,
        direction: Direction,
        ticker: u128,
    ) -> Vec<Box<dyn Object>> {
        let spell = &mut self.spells[self.active_spell];
        self.energy -= spell.cost();
        spell.evoke(self.location, direction, ticker)
    }

    pub fn active_spell_can_evoke(&self, ticker: u128) -> bool {
        !self.spells[self.active_spell].on_cooldown(ticker)
            && self.energy >= self.spells[self.active_spell].cost()
    }
}

impl Unit for Player {
    fn step(&mut self, step: Point) {
        self.location = step;
    }

    fn speed(&self) -> f64 {
        1.0
    }
}
impl Entity for Player {
    fn location(&self) -> Point {
        self.location
    }
}
