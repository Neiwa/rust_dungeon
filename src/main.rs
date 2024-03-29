use command::{AsCommand, Command};
use console::coord::AsDirection;
use console::ConsoleUnit;
use crossterm::{
    cursor,
    event::{self, poll, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Color, Stylize},
    terminal::{self, size, SetSize},
};
use fireball::Fireball;
use point::{AsPoint, Point};
use render_action::RenderAction;

use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::{self, Write},
    time::{Duration, Instant},
};

mod command;
mod console;
mod fireball;
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
    fireballs: Vec<Fireball>,
}

struct Display<'a> {
    status_indicators: HashMap<&'a str, Indicator>,
}

struct Indicator {
    coord: Coord,
    color: Color,
    bg_color: Color,
}
const BG_COLOR: Color = Color::Rgb { r: 4, g: 109, b: 0 };

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

fn queue_unit_draw(stdout: &mut io::Stdout, unit: &dyn ConsoleUnit) -> io::Result<()> {
    queue!(
        stdout,
        cursor::MoveTo(unit.last_coord().x as u16, unit.last_coord().y as u16),
        style::PrintStyledContent(" ".with(BG_COLOR)),
        cursor::MoveTo(unit.coord().x as u16, unit.coord().y as u16),
        style::PrintStyledContent(unit.symbol().with(unit.color()).on(BG_COLOR)),
    )?;

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
            style::PrintStyledContent(" ".with(BG_COLOR).on(BG_COLOR)),
            cursor::MoveTo(new.x as u16, new.y as u16),
            style::PrintStyledContent(symbol.with(color).on(BG_COLOR)),
        )?,
        RenderAction::Remove(coord) => queue!(
            stdout,
            cursor::MoveTo(coord.x as u16, coord.y as u16),
            style::PrintStyledContent(" ".with(BG_COLOR).on(BG_COLOR)),
        )?,
        RenderAction::Create {
            symbol,
            color,
            coord,
        } => queue!(
            stdout,
            cursor::MoveTo(coord.x as u16, coord.y as u16),
            style::PrintStyledContent(symbol.with(color).on(BG_COLOR)),
        )?,
    }

    Ok(())
}

fn queue_monsters_draw(stdout: &mut io::Stdout, state: &State) -> io::Result<()> {
    for monster in &state.monsters {
        queue_unit_draw(stdout, monster)?;
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
        fireballs: Vec::new(),
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
                "ticker",
                Indicator {
                    coord: Coord::new(cols as i32 - 5, rows as i32 - 1),
                    color: Color::White,
                    bg_color: Color::Magenta,
                },
            ),
        ]),
    };

    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    for y in 0..rows {
        for x in 0..cols {
            let draw = match (x, y) {
                (1, 1) => Some('▥'.with(Color::White).on(BG_COLOR)),
                (0, 0) => Some('╔'.magenta()),
                (0, y) if y == rows - 1 => Some('╚'.magenta()),
                (x, 0) if x == cols - 1 => Some('╗'.magenta()),
                (x, y) if x == cols - 1 && y == rows - 1 => Some('╝'.magenta()),
                (0, _) => Some('║'.magenta()),
                (x, _) if x == cols - 1 => Some('║'.magenta()),
                (_, 0) => Some('═'.magenta()),
                (_, y) if y == rows - 1 => Some('═'.magenta()),
                _ => None,
            };

            if let Some(content) = draw {
                queue!(
                    stdout,
                    cursor::MoveTo(x as u16, y as u16),
                    style::PrintStyledContent(content)
                )?;
            } else {
                queue!(
                    stdout,
                    cursor::MoveTo(x as u16, y as u16),
                    style::PrintStyledContent(" ".on(BG_COLOR))
                )?;
            }
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

    queue_unit_draw(stdout, &state.player)?;

    stdout.flush()?;

    let mut last_tick = 0;
    let mut tick = false;
    let mut last_fireball_tick = 0;
    let mut fireball_tick = false;
    let mut last_spawn_tick = 0;
    let mut spawn_tick = true;

    let mut exit = false;
    let mut player_moved = false;
    let mut input_tracker: Vec<KeyCode> = Vec::new();
    let mut missed_move_ticks = 0;

    let mut events = Vec::new();

    loop {
        let elapsed = state.start.elapsed();
        let unit_ticker = elapsed.as_millis() / 200;
        if unit_ticker > last_tick {
            last_tick = unit_ticker;
            tick = true;
            player_moved = false;
        }
        let fireball_ticker = elapsed.as_millis() / 100;
        if fireball_ticker > last_fireball_tick {
            last_fireball_tick = fireball_ticker;
            fireball_tick = true;
        }
        let spawn_ticker = elapsed.as_secs() / 5;
        if spawn_ticker > last_spawn_tick {
            last_spawn_tick = spawn_ticker;
            spawn_tick = true;
        }

        let mut actions: VecDeque<RenderAction> = VecDeque::new();

        if poll(Duration::from_millis(20))? {
            let event = read();
            match event {
                Ok(Event::Key(KeyEvent { code, kind, .. })) => match kind {
                    event::KeyEventKind::Press => {
                        if code.as_command().is_some() {
                            if !input_tracker.contains(&code) {
                                input_tracker.push(code);
                            }
                        } else {
                            match code {
                                KeyCode::Esc => {
                                    exit = true;
                                }
                                _ => {}
                            }
                        }
                    }
                    event::KeyEventKind::Release => {
                        if code.as_command().is_some() {
                            if input_tracker.contains(&code) {
                                input_tracker.retain(|&d| d != code);
                            }
                        }
                    }
                    event::KeyEventKind::Repeat => {}
                },
                _ => {}
            }
            events.push((
                elapsed,
                unit_ticker,
                format!("{:?} {:?}", event?, input_tracker),
            ));
        }

        if !player_moved && input_tracker.len() > 0 {
            let prev_pos = state.player.location.as_coord();

            let mut step = Point::new(0.0, 0.0);

            for input in &input_tracker {
                match input.as_command() {
                    Some(Command::Move(direction)) => step += direction.as_point(),
                    Some(Command::Fireball(direction)) => {
                        let fireball =
                            Fireball::new(state.player.location + direction.as_point(), direction);
                        actions.push_back(RenderAction::Create {
                            symbol: fireball.symbol(),
                            color: fireball.color(),
                            coord: fireball.coord(),
                        });
                        state.fireballs.push(fireball);
                    }
                    _ => {}
                }
            }

            // Move
            if step != Point::new(0.0, 0.0) {
                let next_pos = state.player.location + step.normalize(state.player.speed());
                let next_coord = next_pos.as_coord();

                if next_coord.x > 0
                    && next_coord.x < cols - 1
                    && next_coord.y > 0
                    && next_coord.y < rows - 1
                {
                    state.player.step(next_pos);

                    actions.push_back(RenderAction::Move {
                        symbol: state.player.symbol(),
                        color: state.player.color(),
                        old: prev_pos,
                        new: state.player.coord(),
                    });
                    player_moved = true;
                }
            }
        }

        if fireball_tick {
            fireball_tick = false;

            let fireball_len = state.fireballs.len();

            for fireball_ix in (0..fireball_len).rev() {
                let mut fireball = state.fireballs.remove(fireball_ix);
                let prev_coord = fireball.location.as_coord();
                let new_pos = fireball.location + (fireball.direction.as_point() * fireball.speed);

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
                            actions.push_back(RenderAction::Remove(monster.coord()));
                            actions.push_back(RenderAction::Remove(fireball.coord()));
                            hit = true;
                            break;
                        }
                    }

                    if !hit {
                        fireball.location = new_pos;

                        actions.push_back(RenderAction::Move {
                            symbol: fireball.symbol(),
                            color: fireball.color(),
                            old: prev_coord,
                            new: next_coord,
                        });
                        state.fireballs.push(fireball);
                    }
                } else {
                    actions.push_back(RenderAction::Remove(prev_coord));
                }
            }
        }

        if tick {
            tick = false;

            if !player_moved {
                missed_move_ticks += 1;
                events.push((elapsed, unit_ticker, String::from("MissedMove")));
            }

            let monsters_len = state.monsters.len();

            for monster_ix in (0..monsters_len).rev() {
                let mut monster = state.monsters.remove(monster_ix);
                let new_pos = monster.seek(state.player.location, unit_ticker);

                let mut collision = false;

                for other_ix in 0..(monsters_len - 1) {
                    let other_monster = &state.monsters[other_ix];
                    if other_monster.coord() == new_pos.as_coord() {
                        collision = true;
                        break;
                    }
                }

                if !collision {
                    let prev_pos = monster.coord();
                    monster.step(new_pos);

                    actions.push_back(RenderAction::Move {
                        symbol: monster.symbol(),
                        color: monster.color(),
                        old: prev_pos,
                        new: new_pos.as_coord(),
                    });
                }

                if monster.coord() == state.player.coord() {
                    state.score = 0;
                    exit = true;
                    break;
                }

                state.monsters.push(monster);
            }
        }

        if spawn_tick {
            spawn_tick = false;

            if state.monsters.len() < 3 {
                let monster = Unit::new(Coord::new(4, 4), None, None);

                actions.push_back(RenderAction::Create {
                    symbol: monster.symbol(),
                    color: monster.color(),
                    coord: monster.coord(),
                });

                state.monsters.push(monster);
            }
        }

        if actions.len() > 0 {
            while let Some(action) = actions.pop_front() {
                queue_action_draw(stdout, action)?;
            }

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

            queue_value_draw(
                stdout,
                display.status_indicators.get("ticker"),
                format!("{:>3}", missed_move_ticks),
            )?;

            stdout.flush()?;
        }

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
