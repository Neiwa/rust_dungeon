use command::{AsCommand, Command};
use console::coord::AsDirection;
use console::input_tracker::InputTracker;
use console::{loader, AsSymbol, ConsoleUnit};
use crossterm::{
    cursor,
    event::{self, poll, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Color, Stylize},
    terminal::{self, size, SetSize},
};

use entity::monster::Monster;
use entity::object::Object;
use entity::player::Player;
use point::{AsPoint, Point};
use render_action::RenderAction;

use std::collections::HashSet;

use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::{self, Write},
    time::{Duration, Instant},
};

mod command;
mod console;
mod entity;
mod magic;
pub mod point;
mod render_action;
use crate::console::{coord::*, loader_reverse, AsColor};
use crate::entity::*;

struct State {
    score: i32,
    start: Instant,
    monsters: Vec<Monster>,
    player: Player,
    objects: Vec<Box<dyn Object>>,
}

struct Display<'a> {
    status_indicators: HashMap<&'a str, Indicator>,
}

struct Indicator {
    coord: Coord,
    color: Color,
    bg_color: Color,
}

fn bg_color(_coord: Coord) -> Color {
    let r = (2 + (_coord.x * _coord.y ^ 34348798) % 5) as u8;
    let g = (100 + (_coord.x * _coord.y ^ 2344839) % 15) as u8;
    let b = 0;
    Color::Rgb { r, g, b }
}

fn main() -> io::Result<()> {
    let (cols, rows) = size()?;

    // execute!(
    //     io::stdout(),
    //     SetSize(10, 10),
    //     ScrollUp(5)
    // )?;

    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        event::EnableMouseCapture,
        cursor::Hide,
    )?;
    let score = game(&mut stdout);
    execute!(
        stdout,
        terminal::LeaveAlternateScreen,
        event::DisableMouseCapture,
        cursor::Show
    )?;
    terminal::disable_raw_mode()?;

    execute!(io::stdout(), SetSize(cols, rows))?;

    println!("Game Over! Score: {}", score?);

    Ok(())
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

fn queue_actions_draw<I>(stdout: &mut io::Stdout, render_actions: I) -> io::Result<()>
where
    I: Iterator<Item = RenderAction>,
{
    let mut clear: HashSet<Coord> = HashSet::new();
    let mut skip_clear: HashSet<Coord> = HashSet::new();
    let mut renders = Vec::new();

    for render in render_actions {
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
                stdout,
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
                stdout,
                cursor::MoveTo((2 * coord.x + 1) as u16, (coord.y + 1) as u16),
                style::PrintStyledContent(symbol.with(color).on(bg_color(coord))),
            )?,
        }
    }

    Ok(())
}

fn game(stdout: &mut io::Stdout) -> io::Result<i32> {
    let (t_cols, t_rows) = size()?;
    let cols = (((t_cols - 2) / 2) as i32).clamp(0, 30);
    let rows = ((t_rows - 2) as i32).clamp(0, 30);

    let mut state = State {
        start: Instant::now(),
        score: 0,
        player: Player::new(Coord::new(cols as i32 / 2, rows as i32 / 2), 0),
        monsters: vec![
            Monster::new_simple(Coord::new(cols as i32 / 4, rows as i32 / 4), 0),
            Monster::new(
                Coord::new(cols as i32 / 4 + cols as i32 / 2, rows as i32 / 4),
                0,
                Some(40),
                Some(3.0),
            ),
            Monster::new(
                Coord::new(
                    cols as i32 / 4 + cols as i32 / 2,
                    rows as i32 / 4 + rows as i32 / 2,
                ),
                0,
                Some(150),
                None,
            ),
            Monster::new(
                Coord::new(cols as i32 / 4, rows as i32 / 4 + rows as i32 / 2),
                0,
                Some(200),
                None,
            ),
        ],
        objects: Vec::new(),
    };

    let display = Display {
        status_indicators: HashMap::from([
            (
                "clock",
                Indicator {
                    coord: Coord::new((2 * cols - 7).into(), 0),
                    color: Color::White,
                    bg_color: Color::Magenta,
                },
            ),
            (
                "score",
                Indicator {
                    coord: Coord::new(5, 0),
                    color: Color::White,
                    bg_color: Color::Magenta,
                },
            ),
            (
                "spells",
                Indicator {
                    coord: Coord::new(5, rows as i32 + 1),
                    color: Color::White,
                    bg_color: Color::Magenta,
                },
            ),
            (
                "energy",
                Indicator {
                    coord: Coord::new((2 * cols - 10).into(), rows as i32 + 1),
                    color: Color::White,
                    bg_color: Color::Magenta,
                },
            ),
        ]),
    };

    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    for y in 0..rows + 2 {
        for x in 0..2 * cols + 2 {
            let content = match (x, y) {
                (0, 0) => "╔".magenta(),
                (0, y) if y == rows + 1 => "╚".magenta(),
                (x, 0) if x == 2 * cols + 1 => "╗".magenta(),
                (x, y) if x == 2 * cols + 1 && y == rows + 1 => "╝".magenta(),
                (0, _) => "║".magenta(),
                (x, _) if x == 2 * cols + 1 => "║".magenta(),
                (_, 0) => "═".magenta(),
                (_, y) if y == rows + 1 => "═".magenta(),
                _ => " ".on(bg_color(Coord::new(2 * x + 1, y + 1))),
            };

            queue!(
                stdout,
                cursor::MoveTo(x as u16, y as u16),
                style::PrintStyledContent(content)
            )?;
        }
    }
    queue_value_draw(
        stdout,
        display.status_indicators.get("clock"),
        format!("{:>3}", state.start.elapsed().as_secs()),
    )?;
    queue_value_draw(
        stdout,
        display.status_indicators.get("score"),
        format!("{:>3}", state.score),
    )?;

    queue_actions_draw(
        stdout,
        state
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
            ]),
    )?;

    stdout.flush()?;

    let mut last_spawn_tick = 0;

    let mut exit = false;
    let mut input_tracker = InputTracker::new();

    #[allow(dead_code, unused_mut)]
    let mut events: Vec<(u128, String)> = Vec::new();

    loop {
        if poll(Duration::from_millis(20))? {
            let event = read();
            match event {
                Ok(Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                })) => {
                    exit = true;
                }
                Ok(ok_event) => {
                    input_tracker.register_input_event(ok_event);
                }
                _ => {}
            }
        }
        let elapsed = state.start.elapsed();
        let ticker = elapsed.as_millis();

        let mut render_actions: VecDeque<RenderAction> = VecDeque::new();

        // OBJECTS

        let object_len = state.objects.len();

        for object_ix in (0..object_len).rev() {
            let object = &state.objects[object_ix];
            let prev_coord = object.location().as_coord();
            let new_pos = object.next_location(ticker);

            let next_coord = new_pos.as_coord();

            if prev_coord != next_coord {
                let mut object = state.objects.remove(object_ix);
                if next_coord.x >= 0
                    && next_coord.x < cols
                    && next_coord.y >= 0
                    && next_coord.y < rows
                {
                    let mut hit = false;

                    for monster_ix in 0..state.monsters.len() {
                        if state.monsters[monster_ix].coord() == next_coord {
                            state.score += 1;

                            let monster = state.monsters.remove(monster_ix);
                            render_actions.push_back(RenderAction::Remove {
                                coord: monster.coord(),
                                symbol: monster.symbol(),
                            });
                            render_actions.push_back(RenderAction::Remove {
                                coord: object.coord(),
                                symbol: object.symbol(),
                            });
                            hit = true;
                            break;
                        }
                    }

                    if !hit {
                        object.set_location(new_pos, ticker);

                        render_actions.push_back(RenderAction::Move {
                            symbol: object.symbol(),
                            color: object.color(),
                            old: prev_coord,
                            new: next_coord,
                        });
                        state.objects.push(object);
                    }
                } else {
                    render_actions.push_back(RenderAction::Remove {
                        coord: prev_coord,
                        symbol: object.symbol(),
                    });
                }
            }
        }

        // PLAYER

        let (input_state, cursor_coord) = input_tracker.calculate_state();
        let mouse_coord = Coord::new(
            (cursor_coord.x.saturating_sub(1) / 2).into(),
            cursor_coord.y.saturating_sub(1).into(),
        );

        let mut step: Option<Point> = None;

        for key_state in input_state {
            match key_state.as_command() {
                Some(Command::Move(direction)) => {
                    step = step + direction.as_point();
                }
                Some(Command::Evoke(direction)) => {
                    if state.player.active_spell_can_evoke(ticker) {
                        let mut objects = state
                            .player
                            .active_spell_evoke(direction.as_point(), ticker);

                        while let Some(object) = objects.pop() {
                            let object_coord = object.location().as_coord();

                            if object_coord.x >= 0
                                && object_coord.x < cols
                                && object_coord.y >= 0
                                && object_coord.y < rows
                            {
                                render_actions.push_back(RenderAction::Create {
                                    symbol: object.symbol(),
                                    color: object.color(),
                                    coord: object_coord,
                                });

                                state.objects.push(object);
                            }
                        }
                    }
                }
                Some(Command::EvokeMouse) => {
                    if state.player.active_spell_can_evoke(ticker) {
                        let mut objects = state.player.active_spell_evoke(
                            (mouse_coord.as_point() - state.player.location).normalize(1.0),
                            ticker,
                        );

                        while let Some(object) = objects.pop() {
                            let object_coord = object.location().as_coord();

                            if object_coord.x >= 0
                                && object_coord.x < cols
                                && object_coord.y >= 0
                                && object_coord.y < rows
                            {
                                render_actions.push_back(RenderAction::Create {
                                    symbol: object.symbol(),
                                    color: object.color(),
                                    coord: object_coord,
                                });

                                state.objects.push(object);
                            }
                        }
                    }
                }
                Some(Command::CycleSpell(false)) => {
                    state.player.active_spell =
                        (state.player.active_spell + state.player.spells.len() - 1)
                            % state.player.spells.len()
                }
                Some(Command::CycleSpell(true)) => {
                    state.player.active_spell =
                        (state.player.active_spell + 1) % state.player.spells.len()
                }
                Some(Command::SelectSpell(index)) => {
                    if index < state.player.spells.len() {
                        state.player.active_spell = index;
                    }
                }
                _ => {}
            }
        }

        if let Some(vec) = step {
            let prev_pos = state.player.location.as_coord();
            let next_pos = state.player.next_location(vec, ticker);
            let next_coord = next_pos.as_coord();

            if next_coord.x >= 0 && next_coord.x < cols && next_coord.y >= 0 && next_coord.y < rows
            {
                state.player.set_location(next_pos, ticker);

                render_actions.push_back(RenderAction::Move {
                    symbol: state.player.symbol(),
                    color: state.player.color(),
                    old: prev_pos,
                    new: state.player.coord(),
                });
            }
        } else {
            state.player.charge_energy(ticker);
        }

        // MONSTERS

        let monsters_len = state.monsters.len();

        for monster_ix in (0..monsters_len).rev() {
            let mut monster = state.monsters.remove(monster_ix);
            let prev_pos = monster.coord();
            let mut new_pos = monster.seek(state.player.location, ticker);
            let next_coord = new_pos.as_coord();

            if next_coord != prev_pos {
                let mut collision = false;
                if next_coord.x >= 0
                    && next_coord.x < cols
                    && next_coord.y >= 0
                    && next_coord.y < rows
                {
                    for other_ix in 0..(monsters_len - 1) {
                        let other_monster = &state.monsters[other_ix];
                        if other_monster.coord() == next_coord {
                            collision = true;
                            break;
                        }
                    }

                    if monster.coord() == state.player.coord() {
                        state.score = 0;
                        exit = true;
                        break;
                    }
                } else {
                    collision = true;
                }

                if !collision {
                    render_actions.push_back(RenderAction::Move {
                        symbol: monster.symbol(),
                        color: monster.color(),
                        old: prev_pos,
                        new: next_coord,
                    });
                } else {
                    new_pos = monster.location();
                }
            }
            monster.set_location(new_pos, ticker);
            state.monsters.push(monster);
        }

        // SPAWN MONSTERS

        if state.monsters.len() < 3 && ticker.saturating_sub(last_spawn_tick) >= 5_000 {
            let monster = Monster::new(Coord::new(4, 4), ticker, None, None);

            render_actions.push_back(RenderAction::Create {
                symbol: monster.symbol(),
                color: monster.color(),
                coord: monster.coord(),
            });

            state.monsters.push(monster);

            last_spawn_tick = ticker;
        }

        // DRAWING

        execute!(stdout, terminal::BeginSynchronizedUpdate)?;

        queue_actions_draw(stdout, render_actions.into_iter())?;

        queue_value_draw(
            stdout,
            display.status_indicators.get("clock"),
            format!("{:>3}", elapsed.as_secs()),
        )?;

        queue_value_draw(
            stdout,
            display.status_indicators.get("score"),
            format!("{:>3}", state.score),
        )?;

        queue_spells_draw(
            stdout,
            display.status_indicators.get("spells"),
            &state.player,
            ticker,
        )?;

        queue_value_draw(
            stdout,
            display.status_indicators.get("energy"),
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

        stdout.flush()?;
        execute!(stdout, terminal::EndSynchronizedUpdate)?;

        if state.player.coord() == Coord::new(1, 1) || exit {
            break;
        }
    }

    let mut file = File::create("rust_dungeon.log")?;
    for (ticker, log) in events {
        file.write_fmt(format_args!("{:>3}\t{:}\n", ticker, log))?;
    }

    Ok(state.score)
}
