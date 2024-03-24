use console::ConsoleUnit;
use crossterm::{
    cursor,
    event::{self, poll, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Color, Stylize},
    terminal,
    terminal::{size, SetSize},
};
use std::{
    io::{self, Write},
    time::{Duration, Instant},
};

mod monster;
mod console;
pub mod point;
use crate::console::coord::*;
use crate::monster::*;

struct State {
    score: i32,
    clock_coord: Coord,
    score_coord: Coord,
    start: Instant,
    monsters: Vec<monster::Monster>,
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

fn queue_monsters_draw(stdout: &mut io::Stdout, state: &State) -> io::Result<()> {
    for monster in &state.monsters {
        let unit = monster;
        queue!(
            stdout,
            cursor::MoveTo(monster.last_coord.x as u16, monster.last_coord.y as u16),
            style::PrintStyledContent(" ".black()),
            cursor::MoveTo(monster.coord().x as u16, monster.coord().y as u16),
            style::PrintStyledContent(unit.symbol().with(unit.color())),
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
        score: 100,
        clock_coord: Coord::new(cols as i32 - 5, 0),
        score_coord: Coord::new(5, 0),
        monsters: vec![
            Monster::new(Coord::new(cols as i32 / 4, rows as i32 / 4)),
            Monster::new_complex(
                Coord::new(cols as i32 / 4 + cols as i32 / 2, rows as i32 / 4),
                Some(40),
            ),
            Monster::new_complex(
                Coord::new(
                    cols as i32 / 4 + cols as i32 / 2,
                    rows as i32 / 4 + rows as i32 / 2,
                ),
                Some(500),
            ),
        ],
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
    execute!(
        stdout,
        cursor::MoveTo(pos.x as u16, pos.y as u16),
        style::PrintStyledContent("*".cyan())
    )?;

    let mut last_tick: u128 = 0;
    let mut tick = false;
    let mut exit = false;

    loop {
        let elapsed = state.start.elapsed();
        let ticker = elapsed.as_millis() / 200;
        if ticker > last_tick {
            last_tick = ticker;
            tick = true;
        }

        if tick {
            tick = false;
            state.score -= 1;

            queue_score_draw(stdout, &state)?;

            for monster in &mut state.monsters {
                monster.seek(pos);

                if monster.coord() == pos {
                    state.score = 0;
                    exit = true;
                }
            }

            queue_monsters_draw(stdout, &state)?;
        }

        queue_clock_draw(stdout, &state)?;

        if poll(Duration::from_millis(50))? {
            let prev_pos = pos;
            match read()? {
                Event::Key(KeyEvent {
                    code,
                    kind: event::KeyEventKind::Press,
                    ..
                }) => match code {
                    KeyCode::Left => {
                        if pos.x - 1 > 0 {
                            pos += Direction::Left;
                        }
                    }
                    KeyCode::Right => {
                        if pos.x + 2 < cols as i32 {
                            pos += Direction::Right;
                        }
                    }
                    KeyCode::Up => {
                        if pos.y - 1 > 0 {
                            pos += Direction::Up;
                        }
                    }
                    KeyCode::Down => {
                        if pos.y + 2 < rows as i32 {
                            pos += Direction::Down;
                        }
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
            queue!(
                stdout,
                cursor::MoveTo(prev_pos.x as u16, prev_pos.y as u16),
                style::PrintStyledContent(" ".black()),
                cursor::MoveTo(pos.x as u16, pos.y as u16),
                style::PrintStyledContent("@".cyan())
            )?;
        }
        stdout.flush()?;

        if pos.x == 1 && pos.y == 1 || exit {
            break;
        }
    }

    Ok(state.score)
}
