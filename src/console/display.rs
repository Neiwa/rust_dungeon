use std::{
    collections::{HashMap, HashSet, VecDeque},
    io::{self, Write},
};

use crossterm::{
    cursor, execute, queue,
    style::{self, Color, Stylize},
    terminal,
};

use super::{loader, loader_reverse, AsColor, AsSymbol, ConsoleUnit, Coord};
use crate::{player::Player, point::Point, render_action::RenderAction, State};

pub struct Display<'a> {
    pub status_indicators: HashMap<&'a str, Indicator>,
    top_left: Coord,
    bottom_right: Coord,
    width: u32,
    height: u32,
    resolution: Point,
    stdout: &'a mut io::Stdout,
    render_actions: VecDeque<RenderAction>,
}

fn bg_color(coord: Coord) -> Color {
    let r = (2 + (coord.x * coord.y ^ 34348798) % 5) as u8;
    let g = (100 + (coord.x * coord.y ^ 2344839) % 15) as u8;
    let b = 0;
    Color::Rgb { r, g, b }
}

impl<'a> Display<'a> {
    pub fn new(
        top_left: Coord,
        bottom_right: Coord,
        resolution: Point,
        stdout: &'a mut io::Stdout,
    ) -> Self {
        let top_right = Coord::new(bottom_right.x, top_left.y);
        let bottom_left = Coord::new(top_left.x, bottom_right.y);

        Self {
            stdout,
            top_left,
            bottom_right,
            width: (bottom_right.x - top_left.x) as u32,
            height: (bottom_right.y - top_left.y) as u32,
            resolution,
            status_indicators: HashMap::from([
                ("clock", Indicator::new(top_right + Coord::new(-6, 0))),
                ("score", Indicator::new(top_left + Coord::new(4, 0))),
                ("spells", Indicator::new(bottom_left + Coord::new(4, 0))),
                ("energy", Indicator::new(bottom_right + Coord::new(-9, 0))),
            ]),
            render_actions: VecDeque::new(),
        }
    }

    pub fn enqueue_action(&mut self, action: RenderAction) {
        self.render_actions.push_back(action);
    }

    pub fn draw_initial(&mut self, state: &State) -> io::Result<()> {
        execute!(self.stdout, terminal::Clear(terminal::ClearType::All))?;

        for y in 0..=self.height {
            for x in 0..=self.width {
                let content = match (x, y) {
                    (0, 0) => "╔".magenta(),
                    (0, y) if y == self.height => "╚".magenta(),
                    (x, 0) if x == self.width => "╗".magenta(),
                    (x, y) if x == self.width && y == self.height => "╝".magenta(),
                    (0, _) => "║".magenta(),
                    (x, _) if x == self.width => "║".magenta(),
                    (_, 0) => "═".magenta(),
                    (_, y) if y == self.height => "═".magenta(),
                    _ => " ".on(bg_color(Coord::new(
                        (x as f64 * self.resolution.x) as i32,
                        (y as f64 * self.resolution.y) as i32,
                    ))),
                };

                queue!(
                    self.stdout,
                    cursor::MoveTo(
                        (self.top_left.x as u32 + x) as u16,
                        (self.top_left.y as u32 + y) as u16
                    ),
                    style::PrintStyledContent(content)
                )?;
            }
        }

        let initial_actions = state
            .monsters
            .iter()
            .map(|m| RenderAction::Create {
                symbol: m.symbol(),
                color: m.color(),
                coord: m.coord(),
            })
            .chain([
                RenderAction::Create {
                    symbol: state.player.symbol(),
                    color: state.player.color(),
                    coord: state.player.coord(),
                },
                RenderAction::Create {
                    symbol: '🚪',
                    color: Color::White,
                    coord: Coord::new(1, 1),
                },
            ]);

        for action in initial_actions {
            self.enqueue_action(action);
        }

        self.draw_state(state)?;

        self.stdout.flush()?;

        Ok(())
    }

    pub fn draw(&mut self, state: &State) -> io::Result<()> {
        execute!(self.stdout, terminal::BeginSynchronizedUpdate)?;

        self.draw_actions()?;

        self.draw_state(state)?;

        self.stdout.flush()?;
        execute!(self.stdout, terminal::EndSynchronizedUpdate)?;

        Ok(())
    }

    fn draw_actions(&mut self) -> io::Result<()> {
        let mut clear: HashSet<Coord> = HashSet::new();
        let mut skip_clear: HashSet<Coord> = HashSet::new();
        let mut renders = Vec::new();

        while let Some(render) = self.render_actions.pop_front() {
            match render {
                RenderAction::Move {
                    old,
                    new,
                    symbol,
                    color,
                } => {
                    clear.insert(old);
                    skip_clear.insert(new);
                    renders.push((new, symbol, color));
                }
                RenderAction::Remove { coord, .. } => {
                    clear.insert(coord);
                }
                RenderAction::Create {
                    coord,
                    symbol,
                    color,
                } => {
                    skip_clear.insert(coord);
                    renders.push((coord, symbol, color));
                }
            };
        }

        for coord in clear {
            if !skip_clear.contains(&coord) {
                queue!(
                    self.stdout,
                    cursor::MoveTo((2 * coord.x + 1) as u16, (coord.y + 1) as u16),
                    style::PrintStyledContent(
                        ' '.on(bg_color(Coord::new(coord.x * 2 + 1, coord.y + 1)))
                    ),
                    style::PrintStyledContent(
                        ' '.on(bg_color(Coord::new(coord.x * 2 + 2, coord.y + 1)))
                    ),
                )?;
            }
        }

        for render in renders {
            match render {
                (coord, symbol, color) => queue!(
                    self.stdout,
                    cursor::MoveTo((2 * coord.x + 1) as u16, (coord.y + 1) as u16),
                    style::PrintStyledContent(symbol.with(color).on(bg_color(coord))),
                )?,
            }
        }

        Ok(())
    }

    fn draw_state(&mut self, state: &State) -> io::Result<()> {
        queue_value_draw(
            self.stdout,
            self.status_indicators.get("clock"),
            format!("{:>3}", state.start.elapsed().as_secs()),
        )?;
        queue_value_draw(
            self.stdout,
            self.status_indicators.get("score"),
            format!("{:>3}", state.score),
        )?;

        queue_spells_draw(
            self.stdout,
            self.status_indicators.get("spells"),
            &state.player,
            state.ticker,
        )?;

        queue_value_draw(
            self.stdout,
            self.status_indicators.get("energy"),
            format!(
                "🧪 {:0>3} {}",
                state.player.energy,
                loader(
                    state.player.energy.into(),
                    state.player.max_energy.into(),
                    state.player.max_energy.into()
                )
            ),
        )?;

        Ok(())
    }
}

pub struct Indicator {
    pub coord: Coord,
    pub color: Color,
    pub bg_color: Color,
}

impl Indicator {
    fn new(coord: Coord) -> Self {
        Self {
            coord,
            color: Color::White,
            bg_color: Color::Magenta,
        }
    }
}

fn queue_value_draw(
    stdout: &mut io::Stdout,
    indicator: Option<&Indicator>,
    value: String,
) -> io::Result<()> {
    if indicator.is_none() {
        return Ok(());
    }

    let ind = indicator.unwrap();

    queue!(
        stdout,
        cursor::MoveTo(ind.coord.x as u16, ind.coord.y as u16),
        style::PrintStyledContent(value.with(ind.color).on(ind.bg_color)),
    )?;

    Ok(())
}

fn queue_spells_draw(
    stdout: &mut io::Stdout,
    indicator: Option<&Indicator>,
    player: &Player,
    ticker: u128,
) -> io::Result<()> {
    if indicator.is_none() {
        return Ok(());
    }

    let ind = indicator.unwrap();

    queue!(
        stdout,
        cursor::MoveTo(ind.coord.x as u16, ind.coord.y as u16)
    )?;

    let spell_len = player.spells.len();
    for i in 0..spell_len {
        let spell = &player.spells[i];
        let is_active = i == player.active_spell;

        let (color, bg_color) = match (is_active, player.energy >= spell.cost()) {
            (true, true) => (Color::DarkMagenta, ind.color),
            (true, false) => (Color::DarkGrey, ind.color),
            (false, true) => (ind.color, ind.bg_color),
            (false, false) => (Color::Grey, ind.bg_color),
        };

        if i > 0 {
            queue!(
                stdout,
                style::PrintStyledContent("═".with(ind.bg_color).on(Color::Black))
            )?;
        }
        queue!(
            stdout,
            style::PrintStyledContent(
                spell
                    .get_spell()
                    .as_symbol()
                    .with(spell.get_spell().as_color())
                    .on(bg_color)
            ),
            style::PrintStyledContent(
                format!(
                    "{}",
                    loader_reverse(
                        spell
                            .cooldown()
                            .saturating_sub(spell.remaining_cooldown(ticker)),
                        spell.cooldown(),
                        spell.cooldown()
                    ),
                )
                .with(spell.get_spell().as_color())
                .on(bg_color)
            ),
            style::PrintStyledContent(format!("{:0>2}", spell.cost()).with(color).on(bg_color)),
            style::PrintStyledContent("═".with(ind.bg_color).on(Color::Black))
        )?;
    }

    Ok(())
}
