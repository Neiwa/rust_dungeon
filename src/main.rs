use command::{AsCommand, Command};
use console::{AsCoord, ConsoleUnit, Coord, Display, InputTracker};
use crossterm::{
    cursor,
    event::{self, poll, read, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, size, SetSize},
};

use entity::monster::Monster;
use entity::object::Object;
use entity::player::Player;
use point::{AsPoint, Point};
use render_action::RenderAction;

use std::{
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
use crate::entity::*;

struct State {
    score: i32,
    start: Instant,
    ticker: u128,
    monsters: Vec<Monster>,
    player: Player,
    objects: Vec<Box<dyn Object>>,
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

fn game(stdout: &mut io::Stdout) -> io::Result<i32> {
    let (t_cols, t_rows) = size()?;
    let cols = (((t_cols - 2) / 2) as i32).clamp(0, 30);
    let rows = ((t_rows - 2) as i32).clamp(0, 30);

    let mut display = Display::new(
        Coord::new(0, 0),
        Coord::new(cols * 2 + 1, rows + 1),
        Point::new(2.0, 1.0),
        stdout,
    );

    let mut state = State {
        start: Instant::now(),
        ticker: 0,
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

    display.draw_initial(&state)?;

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
        state.ticker = state.start.elapsed().as_millis();

        // OBJECTS

        let object_len = state.objects.len();

        for object_ix in (0..object_len).rev() {
            let object = &state.objects[object_ix];
            let prev_coord = object.location().as_coord();
            let new_pos = object.next_location(state.ticker);

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
                            display.enqueue_action(RenderAction::Remove {
                                coord: monster.coord(),
                                symbol: monster.symbol(),
                            });
                            display.enqueue_action(RenderAction::Remove {
                                coord: object.coord(),
                                symbol: object.symbol(),
                            });
                            hit = true;
                            break;
                        }
                    }

                    if !hit {
                        object.set_location(new_pos, state.ticker);

                        display.enqueue_action(RenderAction::Move {
                            symbol: object.symbol(),
                            color: object.color(),
                            old: prev_coord,
                            new: next_coord,
                        });
                        state.objects.push(object);
                    }
                } else {
                    display.enqueue_action(RenderAction::Remove {
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
                    if state.player.active_spell_can_evoke(state.ticker) {
                        let mut objects = state
                            .player
                            .active_spell_evoke(direction.as_point(), state.ticker);

                        while let Some(object) = objects.pop() {
                            let object_coord = object.location().as_coord();

                            if object_coord.x >= 0
                                && object_coord.x < cols
                                && object_coord.y >= 0
                                && object_coord.y < rows
                            {
                                display.enqueue_action(RenderAction::Create {
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
                    if state.player.active_spell_can_evoke(state.ticker) {
                        let mut objects = state.player.active_spell_evoke(
                            (mouse_coord.as_point() - state.player.location).normalize(1.0),
                            state.ticker,
                        );

                        while let Some(object) = objects.pop() {
                            let object_coord = object.location().as_coord();

                            if object_coord.x >= 0
                                && object_coord.x < cols
                                && object_coord.y >= 0
                                && object_coord.y < rows
                            {
                                display.enqueue_action(RenderAction::Create {
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
            let next_pos = state.player.next_location(vec, state.ticker);
            let next_coord = next_pos.as_coord();

            if next_coord.x >= 0 && next_coord.x < cols && next_coord.y >= 0 && next_coord.y < rows
            {
                state.player.set_location(next_pos, state.ticker);

                display.enqueue_action(RenderAction::Move {
                    symbol: state.player.symbol(),
                    color: state.player.color(),
                    old: prev_pos,
                    new: state.player.coord(),
                });
            }
        } else {
            state.player.charge_energy(state.ticker);
        }

        // MONSTERS

        let monsters_len = state.monsters.len();

        for monster_ix in (0..monsters_len).rev() {
            let mut monster = state.monsters.remove(monster_ix);
            let prev_pos = monster.coord();
            let mut new_pos = monster.seek(state.player.location, state.ticker);
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
                    display.enqueue_action(RenderAction::Move {
                        symbol: monster.symbol(),
                        color: monster.color(),
                        old: prev_pos,
                        new: next_coord,
                    });
                } else {
                    new_pos = monster.location();
                }
            }
            monster.set_location(new_pos, state.ticker);
            state.monsters.push(monster);
        }

        // SPAWN MONSTERS

        if state.monsters.len() < 3 && state.ticker.saturating_sub(last_spawn_tick) >= 5_000 {
            let monster = Monster::new(Coord::new(4, 4), state.ticker, None, None);

            display.enqueue_action(RenderAction::Create {
                symbol: monster.symbol(),
                color: monster.color(),
                coord: monster.coord(),
            });

            state.monsters.push(monster);

            last_spawn_tick = state.ticker;
        }

        // DRAWING
        display.draw(&state)?;

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
