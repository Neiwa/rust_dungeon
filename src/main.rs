use console::ConsoleUnit;
use crossterm::{
    cursor,
    event::{self, poll, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Color, Stylize},
    terminal::{self, size, SetSize},
};
use std::{
    collections::HashMap,
    io::{self, Write},
    time::{Duration, Instant},
};

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
        return Ok(())
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
            ),
            Unit::new(
                Coord::new(
                    cols as i32 / 4 + cols as i32 / 2,
                    rows as i32 / 4 + rows as i32 / 2,
                ),
                Some(500),
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
        ]),
    };

    state.monsters.push(Unit::new(
        Coord::new(cols as i32 / 4, rows as i32 / 4 + rows as i32 / 2),
        Some(200),
    ));

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

    let mut last_tick: u128 = 0;
    let mut tick = false;
    let mut exit = false;
    let mut player_moved = false;

    loop {
        let elapsed = state.start.elapsed();
        let ticker = elapsed.as_millis() / 200;
        if ticker > last_tick {
            last_tick = ticker;
            tick = true;
            player_moved = false;
        }

        if poll(Duration::from_millis(0))? {
            let mut step: Option<Direction> = None;

            match read()? {
                Event::Key(KeyEvent {
                    code,
                    kind: event::KeyEventKind::Press,
                    ..
                }) => match code {
                    KeyCode::Left => {
                        step = Some(Direction::Left);
                    }
                    KeyCode::Right => {
                        step = Some(Direction::Right);
                    }
                    KeyCode::Up => {
                        step = Some(Direction::Up);
                    }
                    KeyCode::Down => {
                        step = Some(Direction::Down);
                    }
                    KeyCode::Esc => {
                        break;
                    }
                    _ => {
                        continue;
                    }
                },
                _ => {}
            }

            if !player_moved && step.is_some() {
                let prev_pos = state.player.location.as_coord();
                match step {
                    Some(Direction::Left) => {
                        if prev_pos.x - 1 > 0 {
                            state
                                .player
                                .step(state.player.location + Direction::Left.as_point());
                        }
                    }
                    Some(Direction::Right) => {
                        if prev_pos.x + 2 < cols as i32 {
                            state
                                .player
                                .step(state.player.location + Direction::Right.as_point());
                        }
                    }
                    Some(Direction::Up) => {
                        if prev_pos.y - 1 > 0 {
                            state
                                .player
                                .step(state.player.location + Direction::Up.as_point());
                        }
                    }
                    Some(Direction::Down) => {
                        if prev_pos.y + 2 < rows as i32 {
                            state
                                .player
                                .step(state.player.location + Direction::Down.as_point());
                        }
                    }
                    _ => {}
                }

                queue_unit_draw(stdout, &state.player)?;
                queue_value_draw(
                    stdout,
                    display.status_indicators.get("player_pos"),
                    format!("{} {}", state.player.coord().x, state.player.coord().y),
                )?;
                player_moved = true;
            }
        }

        if tick {
            tick = false;
            state.score -= 1;

            queue_value_draw(
                stdout,
                display.status_indicators.get("score"),
                format!("{:>3}", state.score),
            )?;

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
                    monster.step(new_pos);
                }

                if monster.coord() == state.player.coord() {
                    state.score = 0;
                    exit = true;
                    break;
                }

                state.monsters.push(monster);
            }

            queue_monsters_draw(stdout, &state)?;
        }

        queue_value_draw(
            stdout,
            display.status_indicators.get("clock"),
            format!("{:>3}", state.start.elapsed().as_secs()),
        )?;

        stdout.flush()?;

        if state.player.coord() == Coord::new(1, 1) || exit {
            break;
        }
    }

    Ok(state.score)
}
