use console::ConsoleUnit;
use crossterm::{
    cursor,
    event::{self, poll, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Color, Stylize},
    terminal,
    terminal::{size, SetSize},
};
use point::Point;
use std::{
    collections::VecDeque,
    io::{self, Write},
    ops::Deref,
    time::{Duration, Instant},
};

mod console;
pub mod point;
mod unit;
use crate::console::coord::*;
use crate::unit::*;

struct State {
    score: i32,
    clock_coord: Coord,
    score_coord: Coord,
    start: Instant,
    monsters: Vec<unit::Unit>,
    player: Player,
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

fn queue_clock_draw(stdout: &mut io::Stdout, state: &State) -> io::Result<()> {
    queue!(
        stdout,
        cursor::MoveTo(state.clock_coord.x as u16, state.clock_coord.y as u16),
        style::PrintStyledContent(
            format!("{:>3}", state.start.elapsed().as_secs())
                .with(Color::White)
                .on(Color::Magenta)
        ),
    )?;

    Ok(())
}

fn queue_score_draw(stdout: &mut io::Stdout, state: &State) -> io::Result<()> {
    queue!(
        stdout,
        cursor::MoveTo(state.score_coord.x as u16, state.score_coord.y as u16),
        style::PrintStyledContent(
            format!("{:>3}", state.score)
                .with(Color::White)
                .on(Color::Magenta)
        ),
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
        clock_coord: Coord::new(cols as i32 - 5, 0),
        score_coord: Coord::new(5, 0),
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
            }
        }
    }
    queue_clock_draw(stdout, &state)?;
    queue_score_draw(stdout, &state)?;
    queue_monsters_draw(stdout, &state)?;

    stdout.flush()?;

    let mut pos = Coord {
        x: (cols as i32 / 2),
        y: (rows as i32 / 2),
    };

    queue_unit_draw(stdout, &state.player)?;

    let mut last_tick: u128 = 0;
    let mut tick = false;
    let mut exit = false;
    let mut move_queue: VecDeque<Direction> = VecDeque::new();

    loop {
        let elapsed = state.start.elapsed();
        let ticker = elapsed.as_millis() / 200;
        if ticker > last_tick {
            last_tick = ticker;
            tick = true;
        }

        if poll(Duration::from_millis(50))? {
            match read()? {
                Event::Key(KeyEvent {
                    code,
                    kind: event::KeyEventKind::Press,
                    ..
                }) => match code {
                    KeyCode::Left => {
                        if move_queue.len() > 2 {
                            move_queue.pop_back();
                        }
                        move_queue.push_back(Direction::Left);
                    }
                    KeyCode::Right => {
                        if move_queue.len() > 2 {
                            move_queue.pop_back();
                        }
                        move_queue.push_back(Direction::Right);
                    }
                    KeyCode::Up => {
                        if move_queue.len() > 2 {
                            move_queue.pop_back();
                        }
                        move_queue.push_back(Direction::Up);
                    }
                    KeyCode::Down => {
                        if move_queue.len() > 2 {
                            move_queue.pop_back();
                        }
                        move_queue.push_back(Direction::Down);
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
        }

        if tick {
            tick = false;
            state.score -= 1;

            queue_score_draw(stdout, &state)?;

            if move_queue.len() > 0 {
                let prev_pos = pos;
                match move_queue.pop_front() {
                    Some(Direction::Left) => {
                        if pos.x - 1 > 0 {
                            pos += Direction::Left;
                        }
                    }
                    Some(Direction::Right) => {
                        if pos.x + 2 < cols as i32 {
                            pos += Direction::Right;
                        }
                    }
                    Some(Direction::Up) => {
                        if pos.y - 1 > 0 {
                            pos += Direction::Up;
                        }
                    }
                    Some(Direction::Down) => {
                        if pos.y + 2 < rows as i32 {
                            pos += Direction::Down;
                        }
                    }
                    _ => {}
                }

                queue!(
                    stdout,
                    cursor::MoveTo(prev_pos.x as u16, prev_pos.y as u16),
                    style::PrintStyledContent(" ".black()),
                    cursor::MoveTo(pos.x as u16, pos.y as u16),
                    style::PrintStyledContent("@".cyan())
                )?;
            }

            let monsters_len = state.monsters.len();

            for monster_ix in (0..monsters_len).rev() {
                let mut monster = state.monsters.remove(monster_ix);
                let new_pos = monster.seek(state.player.location, ticker);

                let mut collision = false;

                for other_ix in 0..(monsters_len-1) {
                    let other_monster = &state.monsters[other_ix];
                    if other_monster.coord() == monster.coord() {
                        collision = true;
                        break;
                    }
                }

                if !collision {
                    monster.step(new_pos);
                }

                if monster.coord() == pos {
                    state.score = 0;
                    exit = true;
                    break;
                }

                state.monsters.push(monster);
            }

            queue_monsters_draw(stdout, &state)?;
        }

        queue_clock_draw(stdout, &state)?;

        stdout.flush()?;

        if pos.x == 1 && pos.y == 1 || exit {
            break;
        }
    }

    Ok(state.score)
}
