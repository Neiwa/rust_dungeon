use std::collections::{HashSet, VecDeque};

use crossterm::event::{Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use nalgebra::Point2;

use super::input::*;

pub struct InputTracker {
    pressed_keys: HashSet<KeyCode>,
    pressed_mouse_buttons: HashSet<MouseButton>,
    mouse_coord: Point2<u16>,
    current_events: VecDeque<Event>,
    current_state: HashSet<InputState>,
    current_mouse_coord: Point2<u16>,
}

impl InputTracker {
    pub fn new() -> Self {
        Self::new_mouse(Point2::new(0, 0))
    }

    pub fn new_mouse(mouse_location: Point2<u16>) -> Self {
        Self {
            pressed_keys: HashSet::new(),
            pressed_mouse_buttons: HashSet::new(),
            mouse_coord: mouse_location,
            current_events: VecDeque::new(),
            current_state: HashSet::new(),
            current_mouse_coord: mouse_location,
        }
    }

    pub fn register_input_event(&mut self, event: Event) {
        match event {
            Event::Key(_) | Event::Mouse(_) => {
                self.current_events.push_back(event);
            }
            _ => {}
        }
    }

    pub fn calculate_state(&mut self) -> (&HashSet<InputState>, &Point2<u16>) {
        let mut new_state: HashSet<InputState> = HashSet::new();
        let mut still_active_keys = self.pressed_keys.clone();
        let mut still_active_mouse = self.pressed_mouse_buttons.clone();

        while let Some(event) = self.current_events.pop_front() {
            match event {
                Event::Key(key_event) => match key_event.kind {
                    KeyEventKind::Press => {
                        if !self.pressed_keys.contains(&key_event.code) {
                            new_state.insert(InputState::Press(Input::Key(key_event.code)));
                            new_state.insert(InputState::Active(Input::Key(key_event.code)));
                            self.pressed_keys.insert(key_event.code);
                        }
                    }
                    KeyEventKind::Release => {
                        new_state.insert(InputState::Release(Input::Key(key_event.code)));
                        self.pressed_keys.remove(&key_event.code);
                        still_active_keys.remove(&key_event.code);
                    }
                    KeyEventKind::Repeat => todo!(),
                },
                Event::Mouse(mouse_event) => {
                    self.mouse_coord =
                        Point2::new(mouse_event.column.into(), mouse_event.row.into());

                    if let Some(input) = mouse_event.kind.as_input() {
                        match mouse_event.kind {
                            MouseEventKind::Down(key) => {
                                if !self.pressed_mouse_buttons.contains(&key) {
                                    new_state.insert(InputState::Press(input));
                                    new_state.insert(InputState::Active(input));
                                    self.pressed_mouse_buttons.insert(key);
                                }
                            }
                            MouseEventKind::Up(key) => {
                                new_state.insert(InputState::Release(input));
                                self.pressed_mouse_buttons.remove(&key);
                                still_active_mouse.remove(&key);
                            }
                            MouseEventKind::ScrollDown
                            | MouseEventKind::ScrollUp
                            | MouseEventKind::ScrollLeft
                            | MouseEventKind::ScrollRight => {
                                new_state.insert(InputState::Press(input));
                            }
                            MouseEventKind::Moved | MouseEventKind::Drag(_) => {}
                        };
                    }
                }
                _ => {}
            }
        }

        for code in still_active_keys {
            new_state.insert(InputState::Active(Input::Key(code)));
        }
        for button in still_active_mouse {
            if let Some(input) = button.as_input() {
                new_state.insert(InputState::Active(input));
            }
        }

        self.current_state = new_state;
        self.current_mouse_coord = self.mouse_coord;
        (&self.current_state, &self.current_mouse_coord)
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};

    use super::*;

    fn key_press_event(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new_with_kind(
            code,
            KeyModifiers::empty(),
            KeyEventKind::Press,
        ))
    }

    fn key_release_event(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new_with_kind(
            code,
            KeyModifiers::empty(),
            KeyEventKind::Release,
        ))
    }

    #[test]
    fn test_case_1() {
        // Assign
        let mut tracker = InputTracker::new();
        let code = KeyCode::Char('b');

        // Act
        tracker.register_input_event(key_press_event(code));
        let (state, _) = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 2);
        assert!(state.contains(&InputState::Press(Input::Key(code))));
        assert!(state.contains(&InputState::Active(Input::Key(code))));

        // Act
        tracker.register_input_event(key_release_event(code));
        let (state, _) = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 1);
        assert!(state.contains(&InputState::Release(Input::Key(code))));
    }

    #[test]
    fn test_case_2() {
        // Assign
        let mut tracker = InputTracker::new();
        let code = KeyCode::Char('b');

        // Act
        tracker.register_input_event(key_press_event(code));
        let (state, _) = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 2);
        assert!(state.contains(&InputState::Press(Input::Key(code))));
        assert!(state.contains(&InputState::Active(Input::Key(code))));

        // Act
        let (state, _) = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 1);
        assert!(state.contains(&InputState::Active(Input::Key(code))));

        // Act
        tracker.register_input_event(key_release_event(code));
        let (state, _) = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 1);
        assert!(state.contains(&InputState::Release(Input::Key(code))));
    }

    #[test]
    fn test_case_3() {
        // Assign
        let mut tracker = InputTracker::new();
        let code = KeyCode::Char('b');

        // Act
        tracker.register_input_event(key_press_event(code));
        tracker.register_input_event(key_release_event(code));
        let (state, _) = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 3);
        assert!(state.contains(&InputState::Press(Input::Key(code))));
        assert!(state.contains(&InputState::Active(Input::Key(code))));
        assert!(state.contains(&InputState::Release(Input::Key(code))));

        // Act
        let (state, _) = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 0);
    }

    #[test]
    fn test_case_4() {
        // Assign
        let mut tracker = InputTracker::new();
        let code = KeyCode::Char('b');

        // Act
        tracker.register_input_event(key_press_event(code));
        let (state, _) = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 2);
        assert!(state.contains(&InputState::Press(Input::Key(code))));
        assert!(state.contains(&InputState::Active(Input::Key(code))));

        // Act
        tracker.register_input_event(key_release_event(code));
        tracker.register_input_event(key_press_event(code));
        let (state, _) = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 3);
        assert!(state.contains(&InputState::Press(Input::Key(code))));
        assert!(state.contains(&InputState::Active(Input::Key(code))));
        assert!(state.contains(&InputState::Release(Input::Key(code))));

        // Act
        tracker.register_input_event(key_release_event(code));
        let (state, _) = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 1);
        assert!(state.contains(&InputState::Release(Input::Key(code))));
    }

    #[test]
    fn test_case_5() {
        // Assign
        let mut tracker = InputTracker::new();
        let code = KeyCode::Char('b');

        // Act
        tracker.register_input_event(key_press_event(code));
        let (state, _) = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 2);
        assert!(state.contains(&InputState::Press(Input::Key(code))));
        assert!(state.contains(&InputState::Active(Input::Key(code))));

        // Act
        tracker.register_input_event(key_press_event(code));
        tracker.register_input_event(key_release_event(code));
        let (state, _) = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 1);
        assert!(state.contains(&InputState::Release(Input::Key(code))));

        // Act
        let (state, _) = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 0);
    }

    #[test]
    fn test_case_6() {
        // Assign
        let mut tracker = InputTracker::new();
        let code = KeyCode::Char('b');

        // Act
        tracker.register_input_event(key_press_event(code));
        let (state, _) = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 2);
        assert!(state.contains(&InputState::Press(Input::Key(code))));
        assert!(state.contains(&InputState::Active(Input::Key(code))));

        // Act
        tracker.register_input_event(key_press_event(code));
        tracker.register_input_event(key_release_event(code));
        tracker.register_input_event(key_press_event(code));
        let (state, _) = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 3);
        assert!(state.contains(&InputState::Press(Input::Key(code))));
        assert!(state.contains(&InputState::Active(Input::Key(code))));
        assert!(state.contains(&InputState::Release(Input::Key(code))));

        // Act
        let (state, _) = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 1);
        assert!(state.contains(&InputState::Active(Input::Key(code))));
    }

    #[test]
    fn test_case_7() {
        // Assign
        let mut tracker = InputTracker::new();
        let code = KeyCode::Char('b');

        // Act
        tracker.register_input_event(key_press_event(code));
        let (state, _) = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 2);
        assert!(state.contains(&InputState::Press(Input::Key(code))));
        assert!(state.contains(&InputState::Active(Input::Key(code))));

        // Act
        tracker.register_input_event(key_press_event(code));
        let (state, _) = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 1);
        assert!(state.contains(&InputState::Active(Input::Key(code))));

        // Act
        tracker.register_input_event(key_release_event(code));
        let (state, _) = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 1);
        assert!(state.contains(&InputState::Release(Input::Key(code))));
    }
}
