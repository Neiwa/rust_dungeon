use command::{AsCommand, Command};
use console::keyboard_state::KeyboardTracker;
use console::{coord::AsDirection, loader};
use console::{AsSymbol, ConsoleUnit};
use crossterm::{
    cursor,
    event::{self, poll, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Color, Stylize},
    terminal::{self, size, SetSize},
};

use point::{AsPoint, Point};
use render_action::RenderAction;

use std::collections::HashSet;
use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::{self, Write},
    time::{Duration, Instant},
};

use unicode_width::UnicodeWidthChar;

mod command;
mod console;
mod magic;
pub mod point;
mod render_action;
mod unit;
use crate::console::coord::*;
use crate::unit::*;

struct State {
    score: i32,
    start: Instant,
    monsters: Vec<unit::Unit>,
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
        cursor::Hide
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

fn queue_actions_draw(
    stdout: &mut io::Stdout,
    render_actions: &VecDeque<RenderAction>,
) -> io::Result<()> {
    let mut clear: HashSet<Coord> = HashSet::new();
    let mut skip_clear: HashSet<Coord> = HashSet::new();

    for render in render_actions {
        match render {
            RenderAction::Move {
                old, new, symbol, ..
            } => {
                let width = UnicodeWidthChar::width(*symbol).unwrap_or(0);
                for i in 0..width {
                    clear.insert(*old + Coord::new(1, 0) * i);
                }
                for i in 0..width {
                    skip_clear.insert(*new + Coord::new(1, 0) * i);
                }
            }
            RenderAction::Remove { coord, symbol } => {
                for i in 0..UnicodeWidthChar::width(*symbol).unwrap_or(0) {
                    clear.insert(*coord + Coord::new(1, 0) * i);
                }
            }
            RenderAction::Create { coord, symbol, .. } => {
                for i in 0..UnicodeWidthChar::width(*symbol).unwrap_or(0) {
                    skip_clear.insert(*coord + Coord::new(1, 0) * i);
                }
            }
        };
    }

    for coord in clear {
        if !skip_clear.contains(&coord) {
            queue!(
                stdout,
                cursor::MoveTo(coord.x as u16, coord.y as u16),
                style::PrintStyledContent(' '.on(bg_color(coord))),
            )?;
        }
    }

    for render in render_actions {
        match *render {
            RenderAction::Move {
                symbol, color, new, ..
            } => queue!(
                stdout,
                cursor::MoveTo(new.x as u16, new.y as u16),
                style::PrintStyledContent(symbol.with(color).on(bg_color(new))),
            )?,
            RenderAction::Create {
                symbol,
                color,
                coord,
            } => queue!(
                stdout,
                cursor::MoveTo(coord.x as u16, coord.y as u16),
                style::PrintStyledContent(symbol.with(color).on(bg_color(coord))),
            )?,
            _ => {}
        }
    }

    Ok(())
}

fn queue_action_draw(stdout: &mut io::Stdout, action: RenderAction) -> io::Result<()> {
    match action {
        RenderAction::Move {
            symbol,
            color,
            old,
            new,
        } => queue!(
            stdout,
            cursor::MoveTo(old.x as u16, old.y as u16),
            style::PrintStyledContent(" ".on(bg_color(old))),
            cursor::MoveTo(new.x as u16, new.y as u16),
            style::PrintStyledContent(symbol.with(color).on(bg_color(new))),
        )?,
        RenderAction::Remove { coord, .. } => queue!(
            stdout,
            cursor::MoveTo(coord.x as u16, coord.y as u16),
            style::PrintStyledContent(" ".on(bg_color(coord))),
        )?,
        RenderAction::Create {
            symbol,
            color,
            coord,
        } => queue!(
            stdout,
            cursor::MoveTo(coord.x as u16, coord.y as u16),
            style::PrintStyledContent(symbol.with(color).on(bg_color(coord))),
        )?,
    }

    Ok(())
}

fn queue_monsters_draw(stdout: &mut io::Stdout, state: &State) -> io::Result<()> {
    for monster in &state.monsters {
        queue_action_draw(
            stdout,
            RenderAction::Create {
                symbol: monster.symbol(),
                color: monster.color(),
                coord: monster.coord(),
            },
        )?;
    }

    Ok(())
}

fn game(stdout: &mut io::Stdout) -> io::Result<i32> {
    let (t_cols, t_rows) = size()?;
    let cols = (t_cols as i32).clamp(0, 70);
    let rows = (t_rows as i32).clamp(0, 30);

    let mut state = State {
        start: Instant::now(),
        score: 0,
        player: Player::new(Coord::new(cols as i32 / 2, rows as i32 / 2)),
        monsters: vec![
            Unit::new_simple(Coord::new(cols as i32 / 4, rows as i32 / 4)),
            Unit::new(
                Coord::new(cols as i32 / 4 + cols as i32 / 2, rows as i32 / 4),
                Some(40),
                Some(0.7),
            ),
            Unit::new(
                Coord::new(
                    cols as i32 / 4 + cols as i32 / 2,
                    rows as i32 / 4 + rows as i32 / 2,
                ),
                Some(500),
                None,
            ),
            Unit::new(
                Coord::new(cols as i32 / 4, rows as i32 / 4 + rows as i32 / 2),
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
                    coord: Coord::new(cols as i32 - 5, 0),
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
                "player_pos",
                Indicator {
                    coord: Coord::new(5, rows as i32 - 1),
                    color: Color::White,
                    bg_color: Color::Magenta,
                },
            ),
            (
                "energy",
                Indicator {
                    coord: Coord::new(cols as i32 - 20, rows as i32 - 1),
                    color: Color::White,
                    bg_color: Color::Magenta,
                },
            ),
        ]),
    };

    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    for y in 0..rows {
        for x in 0..cols {
            let content = match (x, y) {
                (1, 1) => '▥'.with(Color::White).on(bg_color(Coord::new(x, y))),
                (0, 0) => '╔'.magenta(),
                (0, y) if y == rows - 1 => '╚'.magenta(),
                (x, 0) if x == cols - 1 => '╗'.magenta(),
                (x, y) if x == cols - 1 && y == rows - 1 => '╝'.magenta(),
                (0, _) => '║'.magenta(),
                (x, _) if x == cols - 1 => '║'.magenta(),
                (_, 0) => '═'.magenta(),
                (_, y) if y == rows - 1 => '═'.magenta(),
                _ => ' '.on(bg_color(Coord::new(x, y))),
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
    queue_monsters_draw(stdout, &state)?;

    queue_action_draw(
        stdout,
        RenderAction::Create {
            symbol: state.player.symbol(),
            color: state.player.color(),
            coord: state.player.coord(),
        },
    )?;

    stdout.flush()?;

    let mut last_tick = 0;
    let mut tick = false;
    let mut last_object_tick = 0;
    let mut object_tick = false;
    let mut last_spawn_tick = 0;
    let mut spawn_tick = true;

    let mut exit = false;
    let mut player_moved = false;
    let mut keyboard_tracker = KeyboardTracker::new();

    let mut events = Vec::new();

    loop {
        let elapsed = state.start.elapsed();
        let unit_ticker = elapsed.as_millis() / 200;
        if unit_ticker > last_tick {
            last_tick = unit_ticker;
            tick = true;
            player_moved = false;
        }
        let object_ticker = elapsed.as_millis() / 100;
        if object_ticker > last_object_tick {
            last_object_tick = object_ticker;
            object_tick = true;
        }
        let spawn_ticker = elapsed.as_secs() / 5;
        if spawn_ticker > last_spawn_tick {
            last_spawn_tick = spawn_ticker;
            spawn_tick = true;
        }

        let mut render_actions: VecDeque<RenderAction> = VecDeque::new();

        if poll(Duration::from_millis(20))? {
            let event = read();

            match event {
                Ok(Event::Key(key_event)) => {
                    keyboard_tracker.register_event(key_event);
                    match key_event {
                        KeyEvent {
                            code: KeyCode::Esc, ..
                        } => {
                            exit = true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if object_tick {
            let object_len = state.objects.len();

            for object_ix in (0..object_len).rev() {
                let mut object = state.objects.remove(object_ix);
                let prev_coord = object.location().as_coord();
                let new_pos = object.location() + object.vector();

                let next_coord = new_pos.as_coord();
                if next_coord.x > 0
                    && next_coord.x < cols - 2
                    && next_coord.y > 0
                    && next_coord.y < rows - 1
                {
                    let mut hit = false;

                    for monster_ix in 0..state.monsters.len() {
                        if state.monsters[monster_ix].coord() == new_pos.as_coord() {
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
                        object.set_location(new_pos);

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

        if tick {
            let keyboard_state = keyboard_tracker.calculate_state();

            let mut step = Point::new(0.0, 0.0);

            for key_state in keyboard_state {
                match key_state.as_command() {
                    Some(Command::Move(direction)) => {
                        step += direction.as_point();
                    }
                    Some(Command::Evoke(direction)) => {
                        if state.player.active_spell_can_evoke(unit_ticker) {
                            let mut objects =
                                state.player.active_spell_evoke(direction, unit_ticker);

                            while let Some(object) = objects.pop() {
                                render_actions.push_back(RenderAction::Create {
                                    symbol: object.symbol(),
                                    color: object.color(),
                                    coord: object.location().as_coord(),
                                });

                                state.objects.push(object);
                            }
                        }
                    }
                    Some(Command::CycleSpell) => {
                        state.player.active_spell =
                            (state.player.active_spell + 1) % state.player.spells.len()
                    }
                    _ => {}
                }
            }

            if step != Point::new(0.0, 0.0) {
                let prev_pos = state.player.location.as_coord();
                let next_pos = state.player.location + step.normalize(state.player.speed());
                let next_coord = next_pos.as_coord();

                if next_coord.x > 0
                    && next_coord.x < cols - 2
                    && next_coord.y > 0
                    && next_coord.y < rows - 1
                {
                    state.player.step(next_pos);

                    render_actions.push_back(RenderAction::Move {
                        symbol: state.player.symbol(),
                        color: state.player.color(),
                        old: prev_pos,
                        new: state.player.coord(),
                    });
                    player_moved = true;
                }
            }

            if !player_moved {
                if state.player.energy < state.player.max_energy {
                    state.player.energy += 1;
                }
                events.push((elapsed, unit_ticker, String::from("MissedMove")));
            }

            let monsters_len = state.monsters.len();

            for monster_ix in (0..monsters_len).rev() {
                let mut monster = state.monsters.remove(monster_ix);
                let new_pos = monster.seek(state.player.location, unit_ticker);
                let next_coord = new_pos.as_coord();

                if next_coord.x > 0
                    && next_coord.x < cols - 2
                    && next_coord.y > 0
                    && next_coord.y < rows - 1
                {
                    let mut collision = false;

                    for other_ix in 0..(monsters_len - 1) {
                        let other_monster = &state.monsters[other_ix];
                        if other_monster.coord() == next_coord {
                            collision = true;
                            break;
                        }
                    }

                    if !collision {
                        let prev_pos = monster.coord();
                        monster.step(new_pos);

                        render_actions.push_back(RenderAction::Move {
                            symbol: monster.symbol(),
                            color: monster.color(),
                            old: prev_pos,
                            new: next_coord,
                        });
                    }

                    if monster.coord() == state.player.coord() {
                        state.score = 0;
                        exit = true;
                        break;
                    }
                }

                state.monsters.push(monster);
            }
        }

        if spawn_tick {
            if state.monsters.len() < 3 {
                let monster = Unit::new(Coord::new(4, 4), None, None);

                render_actions.push_back(RenderAction::Create {
                    symbol: monster.symbol(),
                    color: monster.color(),
                    coord: monster.coord(),
                });

                state.monsters.push(monster);
            }
        }

        if tick || spawn_tick || object_tick || render_actions.len() > 0 {
            queue_actions_draw(stdout, &render_actions)?;

            queue_value_draw(
                stdout,
                display.status_indicators.get("player_pos"),
                format!("{:?}", state.player.coord()),
            )?;

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

            let spell = state.player.get_active_spell();

            queue_value_draw(
                stdout,
                display.status_indicators.get("energy"),
                format!(
                    "🧪 {:0>3}  {} {} {:0>2} {}",
                    state.player.energy,
                    loader(
                        spell.cooldown() - spell.remaining_cooldown(unit_ticker),
                        spell.cooldown(),
                        spell.cooldown()
                    ),
                    spell.get_spell().as_symbol(),
                    state.player.energy / spell.cost(),
                    loader(
                        state.player.energy as u128 % spell.cost() as u128,
                        spell.cost() as u128,
                        spell.cost() as u128
                    )
                ),
            )?;

            stdout.flush()?;
        }
        tick = false;
        spawn_tick = false;
        object_tick = false;

        if state.player.coord() == Coord::new(1, 1) || exit {
            break;
        }
    }

    let mut file = File::create("rust_dungeon.log")?;
    for (duration, ticker, log) in events {
        file.write_fmt(format_args!(
            "{:>10}\t{:>3}\t{:}\n",
            duration.as_millis(),
            ticker,
            log
        ))?;
    }

    Ok(state.score)
}
