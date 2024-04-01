use crate::{
    magic::{fireball::FireballMagic, inferno::InfernoMagic, sphere::SphereMagic, Magic},
    object::Object,
    point::Point,
    Coord, Entity, Unit,
};

pub struct Player {
    pub location: Point,
    pub energy: u32,
    pub max_energy: u32,
    pub spells: Vec<Box<dyn Magic>>,
    pub active_spell: usize,
    last_tick: u128,
    last_action_tick: u128,

    energy_recharge_tracker: u128,
}

impl Player {
    pub fn new(coord: Coord, ticker: u128) -> Self {
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
            last_tick: ticker,
            last_action_tick: ticker,
            energy_recharge_tracker: 0,
        }
    }

    #[allow(dead_code)]
    pub fn get_active_spell(&self) -> &Box<dyn Magic> {
        self.spells.get(self.active_spell).unwrap()
    }

    pub fn active_spell_evoke(&mut self, direction: Point, ticker: u128) -> Vec<Box<dyn Object>> {
        let spell = &mut self.spells[self.active_spell];
        self.energy -= spell.cost();
        self.last_action_tick = ticker;
        spell.evoke(self.location, direction, ticker)
    }

    pub fn active_spell_can_evoke(&self, ticker: u128) -> bool {
        !self.spells[self.active_spell].on_cooldown(ticker)
            && self.energy >= self.spells[self.active_spell].cost()
    }

    pub fn next_location(&self, vector: Point, ticker: u128) -> Point {
        self.location
            + vector.normalize(self.speed() / 1000.0) * ticker.saturating_sub(self.last_tick)
    }

    pub fn set_ticker(&mut self, ticker: u128) {
        self.last_tick = ticker;
    }

    pub fn charge_energy(&mut self, ticker: u128) {
        if ticker.saturating_sub(self.last_action_tick) > 1_000 {
            self.energy_recharge_tracker += ticker.saturating_sub(self.last_tick);
            let stored_energy = self.energy_recharge_tracker / 200;

            self.energy = (self.energy + stored_energy as u32).clamp(0, self.max_energy);

            if self.energy < self.max_energy {
                self.energy_recharge_tracker -= stored_energy * 200;
            } else {
                self.energy_recharge_tracker = 0;
            }
        }
        self.last_tick = ticker;
    }
}

impl Unit for Player {
    fn speed(&self) -> f64 {
        5.0
    }

    fn set_location(&mut self, location: Point, ticker: u128) {
        self.location = location;
        self.last_tick = ticker;
        self.last_action_tick = ticker;
    }
}

impl Entity for Player {
    fn location(&self) -> Point {
        self.location
    }
}
