use action::Action;
use console::ConsoleUnit;
use crossterm::{
    cursor,
    event::{self, poll, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Color, Stylize},
    terminal::{self, size, SetSize},
};
use point::Point;
use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::{self, Write},
    time::{Duration, Instant},
};

mod action;
mod console;
pub mod point;
mod unit;
use crate::console::coord::*;
use crate::unit::*;

struct State {
    score: i32,
    start: Instant,
    monsters: Vec<unit::Unit>,
    player: Player,
}

struct Display<'a> {
    status_indicators: HashMap<&'a str, Indicator>,
}

struct Indicator {
    coord: Coord,
    color: Color,
    bg_color: Color,
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

fn queue_unit_draw(stdout: &mut io::Stdout, unit: &dyn ConsoleUnit) -> io::Result<()> {
    queue!(
        stdout,
        cursor::MoveTo(unit.last_coord().x as u16, unit.last_coord().y as u16),
        style::PrintStyledContent(" ".black()),
        cursor::MoveTo(unit.coord().x as u16, unit.coord().y as u16),
        style::PrintStyledContent(unit.symbol().with(unit.color())),
    )?;

    Ok(())
}

fn queue_action_draw(stdout: &mut io::Stdout, action: Action) -> io::Result<()> {
    match action {
        Action::Move {
            symbol,
            color,
            old,
            new,
        } => queue!(
            stdout,
            cursor::MoveTo(old.x as u16, old.y as u16),
            style::PrintStyledContent(" ".black()),
            cursor::MoveTo(new.x as u16, new.y as u16),
            style::PrintStyledContent(symbol.with(color)),
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
        score: 100,
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
            if (y == 0 || y == rows - 1) || (x == 0 || x == cols - 1) {
                queue!(
                    stdout,
                    cursor::MoveTo(x as u16, y as u16),
                    style::PrintStyledContent("█".magenta())
                )?;
            } else if x == 1 && y == 1 {
                queue!(
                    stdout,
                    cursor::MoveTo(x as u16, y as u16),
                    style::PrintStyledContent("#".green())
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
    let mut exit = false;
    let mut player_moved = false;
    let mut input_stack: Vec<Direction> = Vec::new();
    let mut missed_move_ticks = 0;

    let mut events = Vec::new();

    loop {
        let elapsed = state.start.elapsed();
        let ticker = elapsed.as_millis() / 200;
        if ticker > last_tick {
            last_tick = ticker;
            tick = true;
            player_moved = false;
        }

        let mut actions: VecDeque<Action> = VecDeque::new();

        if poll(Duration::from_millis(20))? {
            let event = read();
            match event {
                Ok(Event::Key(KeyEvent { code, kind, .. })) => match kind {
                    event::KeyEventKind::Press => match code {
                        KeyCode::Left => {
                            if !input_stack.contains(&Direction::Left) {
                                input_stack.push(Direction::Left);
                            }
                        }
                        KeyCode::Right => {
                            if !input_stack.contains(&Direction::Right) {
                                input_stack.push(Direction::Right);
                            }
                        }
                        KeyCode::Up => {
                            if !input_stack.contains(&Direction::Up) {
                                input_stack.push(Direction::Up);
                            }
                        }
                        KeyCode::Down => {
                            if !input_stack.contains(&Direction::Down) {
                                input_stack.push(Direction::Down);
                            }
                        }
                        KeyCode::Esc => {
                            break;
                        }
                        _ => {}
                    },
                    event::KeyEventKind::Release => match code {
                        KeyCode::Left => {
                            if input_stack.contains(&Direction::Left) {
                                input_stack.retain(|&d| d != Direction::Left);
                            }
                        }
                        KeyCode::Right => {
                            if input_stack.contains(&Direction::Right) {
                                input_stack.retain(|&d| d != Direction::Right);
                            }
                        }
                        KeyCode::Up => {
                            if input_stack.contains(&Direction::Up) {
                                input_stack.retain(|&d| d != Direction::Up);
                            }
                        }
                        KeyCode::Down => {
                            if input_stack.contains(&Direction::Down) {
                                input_stack.retain(|&d| d != Direction::Down);
                            }
                        }
                        _ => {}
                    },
                    event::KeyEventKind::Repeat => {}
                },
                _ => {}
            }
            events.push((elapsed, ticker, format!("{:?} {:?}", event?, input_stack)));
        }

        if !player_moved && input_stack.len() > 0 {
            let prev_pos = state.player.location.as_coord();

            let mut step = input_stack.last().unwrap().as_point();

            if let Some(secondary_direction) = input_stack.get(input_stack.len().wrapping_sub(2)) {
                step += secondary_direction.as_point();
            }

            let next_pos = state.player.location + step.normalize(state.player.speed());
            let next_coord = next_pos.as_coord();
            if next_coord.x > 0
                && next_coord.x < cols - 1
                && next_coord.y > 0
                && next_coord.y < rows - 1
            {
                state.player.step(next_pos);

                actions.push_back(Action::Move {
                    symbol: state.player.symbol(),
                    color: state.player.color(),
                    old: prev_pos,
                    new: state.player.coord(),
                });
                player_moved = true;
            }
        }

        if tick {
            tick = false;
            state.score -= 1;

            if !player_moved {
                missed_move_ticks += 1;
                events.push((elapsed, ticker, String::from("MissedMove")));
            }

            let monsters_len = state.monsters.len();

            for monster_ix in (0..monsters_len).rev() {
                let mut monster = state.monsters.remove(monster_ix);
                let new_pos = monster.seek(state.player.location, ticker);

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

                    actions.push_back(Action::Move {
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
