use std::{
    collections::{HashMap, HashSet, VecDeque},
    io,
};

use crossterm::{
    cursor, execute,
    style::{self, Color, Stylize},
    terminal,
};

use nalgebra::{vector, Point2, Scale2, Vector2};

use super::{loader, loader_reverse, AsColor, AsSymbol, ConsoleUnit};
use crate::{display::Display, player::Player, render_action::RenderAction, Entity, State};

pub struct ConsoleDisplay<'a> {
    pub status_indicators: HashMap<&'a str, Indicator>,
    top_left: Point2<u16>,
    dimensions: Vector2<u16>,
    resolution: Scale2<u16>,
    stdout: &'a mut io::Stdout,
    render_actions: VecDeque<RenderAction>,
}

pub struct Indicator {
    pub coord: Point2<u16>,
    pub color: Color,
    pub bg_color: Color,
}

impl Indicator {
    fn new(coord: Point2<u16>) -> Self {
        Self {
            coord,
            color: Color::White,
            bg_color: Color::Magenta,
        }
    }
}

fn bg_color(coord: Point2<u16>) -> Color {
    let r = (2 + (coord.x * coord.y ^ 3498) % 5) as u8;
    let g = (100 + (coord.x * coord.y ^ 2839) % 15) as u8;
    let b = 0;
    Color::Rgb { r, g, b }
}

trait AsPoint2 {
    fn as_point2(&self) -> Point2<u16>;
}

impl AsPoint2 for Point2<f64> {
    fn as_point2(&self) -> Point2<u16> {
        Point2::new(
            self.x.round().clamp(u16::MIN.into(), u16::MAX.into()) as u16,
            self.y.round().clamp(u16::MIN.into(), u16::MAX.into()) as u16,
        )
    }
}

impl<'a> ConsoleDisplay<'a> {
    pub fn new(
        top_left: Point2<u16>,
        dimensions: Vector2<u16>,
        resolution: Scale2<u16>,
        stdout: &'a mut io::Stdout,
    ) -> Self {
        let bottom_right = top_left + dimensions;
        let top_right = top_left + Vector2::new(dimensions.x, 0);
        let bottom_left = top_left + Vector2::new(0, dimensions.y);

        Self {
            stdout,
            top_left,
            dimensions,
            resolution,
            status_indicators: HashMap::from([
                ("clock", Indicator::new(top_right - Vector2::new(6, 0))),
                ("score", Indicator::new(top_left + Vector2::new(4, 0))),
                ("spells", Indicator::new(bottom_left + Vector2::new(4, 0))),
                ("energy", Indicator::new(bottom_right - Vector2::new(9, 0))),
            ]),
            render_actions: VecDeque::new(),
        }
    }

    fn draw_actions(&mut self) -> io::Result<()> {
        let mut clear: HashSet<Point2<u16>> = HashSet::new();
        let mut skip_clear: HashSet<Point2<u16>> = HashSet::new();
        let mut renders = Vec::new();

        while let Some(render) = self.render_actions.pop_front() {
            match render {
                RenderAction::Move {
                    old,
                    new,
                    symbol,
                    color,
                } => {
                    clear.insert(old.as_point2());
                    skip_clear.insert(new.as_point2());
                    renders.push((new.as_point2(), symbol, color));
                }
                RenderAction::Remove { coord, .. } => {
                    clear.insert(coord.as_point2());
                }
                RenderAction::Create {
                    location: coord,
                    symbol,
                    color,
                } => {
                    skip_clear.insert(coord.as_point2());
                    renders.push((coord.as_point2(), symbol, color));
                }
            };
        }

        for coord in clear {
            if !skip_clear.contains(&coord) {
                let spot = self.resolution * coord
                    + (self.top_left - Point2::new(0, 0) + Vector2::new(1, 1));
                execute!(
                    self.stdout,
                    cursor::MoveTo(spot.x, spot.y),
                    style::PrintStyledContent(' '.on(bg_color(spot))),
                    style::PrintStyledContent(' '.on(bg_color(spot + vector![1, 0]))),
                )?;
            }
        }

        for render in renders {
            match render {
                (coord, symbol, color) => {
                    let spot = self.resolution * coord
                        + (self.top_left - Point2::new(0, 0) + Vector2::new(1, 1));
                    execute!(
                        self.stdout,
                        cursor::MoveTo(spot.x, spot.y),
                        style::PrintStyledContent(symbol.with(color).on(bg_color(spot))),
                    )?;
                }
            }
        }

        Ok(())
    }

    fn draw_state(&mut self, state: &State) -> io::Result<()> {
        draw_value(
            self.stdout,
            self.status_indicators.get("clock"),
            format!("{:>3}", state.ticker / 1000),
        )?;
        draw_value(
            self.stdout,
            self.status_indicators.get("score"),
            format!("{:>3}", state.score),
        )?;

        draw_spells(
            self.stdout,
            self.status_indicators.get("spells"),
            &state.player,
            state.ticker,
        )?;

        draw_value(
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

impl Display for ConsoleDisplay<'_> {
    fn enqueue_action(&mut self, action: RenderAction) {
        self.render_actions.push_back(action);
    }

    fn draw_initial(&mut self, state: &State) -> io::Result<()> {
        execute!(self.stdout, terminal::BeginSynchronizedUpdate)?;
        execute!(self.stdout, terminal::Clear(terminal::ClearType::All))?;

        let (width, height) = (self.dimensions.x, self.dimensions.y);

        for y in 0..=height {
            for x in 0..=width {
                let content = match (x, y) {
                    (0, 0) => "╔".magenta(),
                    (0, y) if y == height => "╚".magenta(),
                    (x, 0) if x == width => "╗".magenta(),
                    (x, y) if x == width && y == height => "╝".magenta(),
                    (0, _) => "║".magenta(),
                    (x, _) if x == width => "║".magenta(),
                    (_, 0) => "═".magenta(),
                    (_, y) if y == height => "═".magenta(),
                    _ => " ".on(bg_color(Point2::new(x, y))),
                };
                let spot = self.top_left + vector!(x, y);
                execute!(
                    self.stdout,
                    cursor::MoveTo(spot.x, spot.y),
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
                location: m.location(),
            })
            .chain([
                RenderAction::Create {
                    symbol: state.player.symbol(),
                    color: state.player.color(),
                    location: state.player.location(),
                },
                RenderAction::Create {
                    symbol: '🚪',
                    color: Color::White,
                    location: Point2::new(1., 1.),
                },
            ]);

        for action in initial_actions {
            self.enqueue_action(action);
        }

        self.draw_actions()?;

        self.draw_state(state)?;

        execute!(self.stdout, terminal::EndSynchronizedUpdate)?;

        Ok(())
    }

    fn draw(&mut self, state: &State) -> io::Result<()> {
        execute!(self.stdout, terminal::BeginSynchronizedUpdate)?;

        self.draw_actions()?;

        self.draw_state(state)?;

        execute!(self.stdout, terminal::EndSynchronizedUpdate)?;

        Ok(())
    }
}

fn draw_value(
    stdout: &mut io::Stdout,
    indicator: Option<&Indicator>,
    value: String,
) -> io::Result<()> {
    if indicator.is_none() {
        return Ok(());
    }

    let ind = indicator.unwrap();

    execute!(
        stdout,
        cursor::MoveTo(ind.coord.x, ind.coord.y),
        style::PrintStyledContent(value.with(ind.color).on(ind.bg_color)),
    )?;

    Ok(())
}

fn draw_spells(
    stdout: &mut io::Stdout,
    indicator: Option<&Indicator>,
    player: &Player,
    ticker: u128,
) -> io::Result<()> {
    if indicator.is_none() {
        return Ok(());
    }

    let ind = indicator.unwrap();

    execute!(stdout, cursor::MoveTo(ind.coord.x, ind.coord.y))?;

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
            execute!(
                stdout,
                style::PrintStyledContent("═".with(ind.bg_color).on(Color::Black))
            )?;
        }
        execute!(
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
