use command::{AsCommand, Command};
use console::{AsVector2, ConsoleDisplay, ConsoleUnit, InputTracker};
use crossterm::{
    cursor,
    event::{self, poll, read, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, size, SetSize},
};

use display::Display;
use entity::monster::Monster;
use entity::object::Object;
use entity::player::Player;
use nalgebra::{Point2, Scale2, Vector2};
use render_action::RenderAction;

use std::{
    fs::File,
    io::{self, Write},
    time::{Duration, Instant},
};

mod command;
mod console;
mod display;
mod entity;
mod magic;
mod render_action;
use crate::entity::*;

struct State {
    score: i32,
    ticker: u128,
    monsters: Vec<Monster>,
    player: Player,
    objects: Vec<Box<dyn Object>>,
}

trait AsCoord {
    fn as_coord(&self) -> Point2<i32>;
}

impl AsCoord for Point2<f64> {
    fn as_coord(&self) -> Point2<i32> {
        Point2::new(self.x as i32, self.y as i32)
    }
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

    let mut display = ConsoleDisplay::new(
        Point2::new(0, 0),
        Vector2::new((cols * 2 + 1) as u16, (rows + 1) as u16),
        Scale2::new(2, 1),
        stdout,
    );

    let timer = Instant::now();

    let mut state = State {
        ticker: 0,
        score: 0,
        player: Player::new(Point2::new(cols as f64 / 2.0, rows as f64 / 2.0), 0),
        monsters: vec![
            Monster::new_simple(Point2::new(cols as f64 / 4.0, rows as f64 / 4.0), 0),
            Monster::new(
                Point2::new(cols as f64 / 4.0 + cols as f64 / 2.0, rows as f64 / 4.0),
                0,
                Some(40),
                Some(3.0),
            ),
            Monster::new(
                Point2::new(
                    cols as f64 / 4.0 + cols as f64 / 2.0,
                    rows as f64 / 4.0 + rows as f64 / 2.0,
                ),
                0,
                Some(150),
                None,
            ),
            Monster::new(
                Point2::new(cols as f64 / 4.0, rows as f64 / 4.0 + rows as f64 / 2.0),
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
        state.ticker = timer.elapsed().as_millis();

        // OBJECTS

        let object_len = state.objects.len();

        for object_ix in (0..object_len).rev() {
            let object = &state.objects[object_ix];
            let old_pos = object.location();
            let new_pos = object.next_location(state.ticker);

            let old_coord = old_pos.as_coord();
            let next_coord = new_pos.as_coord();

            if old_coord != next_coord {
                let mut object = state.objects.remove(object_ix);
                if next_coord.x >= 0
                    && next_coord.x < cols
                    && next_coord.y >= 0
                    && next_coord.y < rows
                {
                    let mut hit = false;

                    for monster_ix in 0..state.monsters.len() {
                        if state.monsters[monster_ix].location().as_coord() == next_coord {
                            state.score += 1;

                            let monster = state.monsters.remove(monster_ix);
                            display.enqueue_action(RenderAction::Remove {
                                coord: monster.location(),
                                symbol: monster.symbol(),
                            });
                            display.enqueue_action(RenderAction::Remove {
                                coord: object.location(),
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
                            old: old_pos,
                            new: new_pos,
                        });
                        state.objects.push(object);
                    }
                } else {
                    display.enqueue_action(RenderAction::Remove {
                        coord: old_pos,
                        symbol: object.symbol(),
                    });
                }
            }
        }

        // PLAYER

        let (input_state, cursor_coord) = input_tracker.calculate_state();
        let mouse_coord = Point2::new(
            (cursor_coord.x.saturating_sub(1) / 2).into(),
            cursor_coord.y.saturating_sub(1).into(),
        );

        let mut step: Vector2<f64> = Vector2::zeros();

        for key_state in input_state {
            match key_state.as_command() {
                Some(Command::Move(direction)) => {
                    step = step + direction.as_vector();
                }
                Some(Command::Evoke(direction)) => {
                    if state.player.active_spell_can_evoke(state.ticker) {
                        let mut objects = state
                            .player
                            .active_spell_evoke(direction.as_vector(), state.ticker);

                        while let Some(object) = objects.pop() {
                            let location = object.location();

                            if location.x >= 0.0
                                && location.x < cols as f64
                                && location.y >= 0.0
                                && location.y < rows as f64
                            {
                                display.enqueue_action(RenderAction::Create {
                                    symbol: object.symbol(),
                                    color: object.color(),
                                    location,
                                });

                                state.objects.push(object);
                            }
                        }
                    }
                }
                Some(Command::EvokeMouse) => {
                    if state.player.active_spell_can_evoke(state.ticker) {
                        let mut objects = state.player.active_spell_evoke(
                            (mouse_coord - state.player.location).normalize(),
                            state.ticker,
                        );

                        while let Some(object) = objects.pop() {
                            let location = object.location();

                            if location.x >= 0.0
                                && location.x < cols as f64
                                && location.y >= 0.0
                                && location.y < rows as f64
                            {
                                display.enqueue_action(RenderAction::Create {
                                    symbol: object.symbol(),
                                    color: object.color(),
                                    location,
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

        if step != Vector2::zeros() {
            let prev_pos = state.player.location();
            let next_pos = state.player.next_location(step, state.ticker);

            if next_pos.x >= 0.0
                && next_pos.x < cols as f64
                && next_pos.y >= 0.0
                && next_pos.y < rows as f64
            {
                state.player.set_location(next_pos, state.ticker);

                display.enqueue_action(RenderAction::Move {
                    symbol: state.player.symbol(),
                    color: state.player.color(),
                    old: prev_pos,
                    new: state.player.location(),
                });
            }
        } else {
            state.player.charge_energy(state.ticker);
        }

        // MONSTERS

        let monsters_len = state.monsters.len();

        for monster_ix in (0..monsters_len).rev() {
            let mut monster = state.monsters.remove(monster_ix);
            let old_pos = monster.location();
            let mut new_pos = monster.seek(state.player.location, state.ticker);

            let old_coord = old_pos.as_coord();
            let next_coord = new_pos.as_coord();

            if next_coord != old_coord {
                let mut collision = false;
                if next_coord.x >= 0
                    && next_coord.x < cols
                    && next_coord.y >= 0
                    && next_coord.y < rows
                {
                    for other_ix in 0..(monsters_len - 1) {
                        let other_monster = &state.monsters[other_ix];
                        if other_monster.location().as_coord() == next_coord {
                            collision = true;
                            break;
                        }
                    }

                    if monster.location().as_coord() == state.player.location().as_coord() {
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
                        old: old_pos,
                        new: new_pos,
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
            let monster = Monster::new(Point2::new(4.0, 4.0), state.ticker, None, None);

            display.enqueue_action(RenderAction::Create {
                symbol: monster.symbol(),
                color: monster.color(),
                location: monster.location(),
            });

            state.monsters.push(monster);

            last_spawn_tick = state.ticker;
        }

        // DRAWING
        display.draw(&state)?;

        if state.player.location().as_coord() == Point2::<i32>::new(1, 1) || exit {
            break;
        }
    }

    let mut file = File::create("rust_dungeon.log")?;
    for (ticker, log) in events {
        file.write_fmt(format_args!("{:>3}\t{:}\n", ticker, log))?;
    }

    Ok(state.score)
}
