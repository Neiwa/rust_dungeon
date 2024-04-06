use command::{AsCommand, Command};
use console::{AsVector2, ConsoleDisplay, ConsoleUnit, InputTracker};
use crossterm::{
    cursor,
    event::{self, poll, read, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{self, size, SetSize},
};

use display::Display;
use entity::monster::Monster;
use entity::object::Object;
use entity::player::Player;
use log::{info, trace, LevelFilter};
use nalgebra::{Point2, Scale2, Vector2};
use render_action::RenderAction;
use simplelog::{format_description, ConfigBuilder, WriteLogger};

use std::{
    fs::File,
    io,
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
        Point2::new(
            self.x.round().clamp(i32::MIN.into(), i32::MAX.into()) as i32,
            self.y.round().clamp(i32::MIN.into(), i32::MAX.into()) as i32,
        )
    }
}

fn main() -> io::Result<()> {
    WriteLogger::init(
        LevelFilter::Trace,
        ConfigBuilder::new()
            .set_thread_level(LevelFilter::Off)
            .set_time_format_custom(format_description!(
                "[hour]:[minute]:[second].[subsecond digits:3]"
            ))
            .set_time_offset_to_local()
            .unwrap()
            .build(),
        File::create("rust_dungeon.log").unwrap(),
    )
    .unwrap();

    info!("Running");

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

    let bounds = Vector2::<f64>::new(cols.into(), rows.into());

    let mut display = ConsoleDisplay::new(
        Point2::new(0, 0),
        Vector2::new((cols * 2 + 3) as u16, (rows + 2) as u16),
        Scale2::new(2, 1),
        stdout,
    );

    let timer = Instant::now();

    let mut state = State {
        ticker: 0,
        score: 0,
        player: Player::new(Point2::new(bounds.x / 2.0, bounds.y / 2.0), 0),
        monsters: vec![
            Monster::new_simple(Point2::new(bounds.x / 4.0, bounds.y / 4.0), 0),
            Monster::new(
                Point2::new(bounds.x * 3.0 / 4.0, bounds.y / 4.0),
                0,
                Some(40),
                Some(3.0),
            ),
            Monster::new(
                Point2::new(bounds.x * 3.0 / 4.0, bounds.y * 3.0 / 4.0),
                0,
                Some(150),
                None,
            ),
            Monster::new(
                Point2::new(bounds.x / 4.0, bounds.y * 3.0 / 4.0),
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
    let mut pause: Option<u128> = None;
    let mut pause_ticker = 0;
    let mut input_tracker = InputTracker::new();

    loop {
        if poll(Duration::from_millis(20))? {
            let event = read();
            match event {
                Ok(Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                })) => {
                    exit = true;
                }
                Ok(Event::Key(KeyEvent {
                    code: KeyCode::Home,
                    kind: KeyEventKind::Release,
                    ..
                })) => {
                    if pause.is_none() {
                        pause = Some(timer.elapsed().as_millis());
                    } else {
                        pause_ticker += timer.elapsed().as_millis() - pause.unwrap();
                        pause = None;
                    }
                }
                Ok(ok_event) => {
                    input_tracker.register_input_event(ok_event);
                }
                _ => {}
            }
        }
        if pause.is_some() {
            continue;
        }

        state.ticker = timer.elapsed().as_millis() - pause_ticker;

        // OBJECTS

        let object_len = state.objects.len();

        for object_ix in (0..object_len).rev() {
            let object = &state.objects[object_ix];
            let old_pos = object.location();
            let next_pos = object.next_location(state.ticker);

            let old_coord = old_pos.as_coord();
            let next_coord = next_pos.as_coord();

            if old_coord != next_coord {
                let mut object = state.objects.remove(object_ix);
                if next_pos.x > 0.0
                    && next_pos.x < bounds.x
                    && next_pos.y > 0.0
                    && next_pos.y < bounds.y
                {
                    let mut hit = false;

                    for monster_ix in 0..state.monsters.len() {
                        if (state.monsters[monster_ix].location() - next_pos).magnitude() < 1.0 {
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
                        object.set_location(next_pos, state.ticker);

                        display.enqueue_action(RenderAction::Move {
                            symbol: object.symbol(),
                            color: object.color(),
                            old: old_pos,
                            new: next_pos,
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
                    step += direction.as_vector();
                }
                Some(Command::Evoke(direction)) => {
                    if state.player.active_spell_can_evoke(state.ticker) {
                        let mut objects = state
                            .player
                            .active_spell_evoke(direction.as_vector(), state.ticker);

                        while let Some(object) = objects.pop() {
                            let location = object.location();

                            if location.x > 0.0
                                && location.x < bounds.x
                                && location.y > 0.0
                                && location.y < bounds.y
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

                            if location.x > 0.0
                                && location.x < bounds.x
                                && location.y > 0.0
                                && location.y < bounds.y
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

            if next_pos.x > 0.0
                && next_pos.x < bounds.x
                && next_pos.y > 0.0
                && next_pos.y < bounds.y
            {
                trace!("Player at {:?} {:?}", next_pos.as_coord(), next_pos);
                state.player.set_location(next_pos, state.ticker);

                display.enqueue_action(RenderAction::Move {
                    symbol: state.player.symbol(),
                    color: state.player.color(),
                    old: prev_pos,
                    new: state.player.location(),
                });
            } else {
                state.player.set_ticker(state.ticker);
            }
        } else {
            state.player.charge_energy(state.ticker);
        }

        // MONSTERS

        let monsters_len = state.monsters.len();

        for monster_ix in (0..monsters_len).rev() {
            let mut monster = state.monsters.remove(monster_ix);
            let old_pos = monster.location();

            if let Some(mut next_pos) = monster.seek(state.player.location(), state.ticker) {
                let old_coord = old_pos.as_coord();
                let next_coord = next_pos.as_coord();

                if next_coord != old_coord {
                    let mut collision = false;
                    if next_pos.x > 0.0
                        && next_pos.x < bounds.x
                        && next_pos.y > 0.0
                        && next_pos.y < bounds.y
                    {
                        for other_ix in 0..(monsters_len - 1) {
                            let other_monster = &state.monsters[other_ix];
                            if (other_monster.location() - next_pos).magnitude() < 1.2 {
                                collision = true;
                                break;
                            }
                        }

                        if (monster.location() - state.player.location()).magnitude() < 1.0 {
                            state.score = 0;
                            exit = true;
                        }
                    } else {
                        collision = true;
                    }

                    if !collision {
                        display.enqueue_action(RenderAction::Move {
                            symbol: monster.symbol(),
                            color: monster.color(),
                            old: old_pos,
                            new: next_pos,
                        });
                    } else {
                        next_pos = old_pos;
                    }
                }
                monster.set_location(next_pos, state.ticker);
            } else {
                monster.set_ticker(state.ticker);
            }
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

        if (state.player.location() - Point2::new(1.0, 1.0)).magnitude() < 1.0 || exit {
            break;
        }
    }

    Ok(state.score)
}
