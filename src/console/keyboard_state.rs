use std::collections::{HashSet, VecDeque};

use crossterm::event::{KeyCode, KeyEvent};

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub enum KeyboardState {
    Press(KeyCode),
    Release(KeyCode),
    Active(KeyCode),
}

pub struct KeyboardTracker {
    active_input: Vec<KeyCode>,
    current_events: VecDeque<KeyEvent>,
    current_state: HashSet<KeyboardState>,
}

impl KeyboardTracker {
    pub fn new() -> Self {
        Self {
            active_input: Vec::new(),
            current_events: VecDeque::new(),
            current_state: HashSet::new(),
        }
    }

    pub fn register_event(&mut self, event: KeyEvent) {
        self.current_events.push_back(event);
    }

    pub fn calculate_state(&mut self) -> &HashSet<KeyboardState> {
        let mut new_state = HashSet::new();

        for state in &self.current_state {
            match state {
                KeyboardState::Active(code) => {
                    if !self.current_state.contains(&KeyboardState::Release(*code)) {
                        new_state.insert(*state);
                    }
                }
                _ => {}
            }
        }

        while let Some(event) = self.current_events.pop_front() {
            match event.kind {
                crossterm::event::KeyEventKind::Press => {
                    if !new_state.contains(&KeyboardState::Active(event.code)) {
                        new_state.insert(KeyboardState::Press(event.code));
                        new_state.insert(KeyboardState::Active(event.code));
                    }
                }
                crossterm::event::KeyEventKind::Release => {
                    if !new_state.contains(&KeyboardState::Press(event.code)) {
                        new_state.remove(&KeyboardState::Active(event.code));
                    }
                    new_state.insert(KeyboardState::Release(event.code));
                }
                crossterm::event::KeyEventKind::Repeat => todo!(),
            }
        }

        self.current_state = new_state;
        &self.current_state
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyEventKind, KeyEventState, KeyModifiers};

    use super::*;

    #[test]
    fn test_case_1() {
        // Assign
        let mut tracker = KeyboardTracker::new();
        let code = KeyCode::Char('b');
        let press = KeyEvent::new_with_kind(
            KeyCode::Char('b'),
            KeyModifiers::empty(),
            KeyEventKind::Press,
        );
        let release = KeyEvent::new_with_kind(
            KeyCode::Char('b'),
            KeyModifiers::empty(),
            KeyEventKind::Release,
        );

        // Act
        tracker.register_event(press);
        let state = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 2);
        assert!(state.contains(&KeyboardState::Press(code)));
        assert!(state.contains(&KeyboardState::Active(code)));

        // Act
        tracker.register_event(release);
        let state = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 1);
        assert!(state.contains(&KeyboardState::Release(code)));
    }

    #[test]
    fn test_case_2() {
        // Assign
        let mut tracker = KeyboardTracker::new();
        let code = KeyCode::Char('b');
        let press = KeyEvent::new_with_kind(
            KeyCode::Char('b'),
            KeyModifiers::empty(),
            KeyEventKind::Press,
        );
        let release = KeyEvent::new_with_kind(
            KeyCode::Char('b'),
            KeyModifiers::empty(),
            KeyEventKind::Release,
        );

        // Act
        tracker.register_event(press);
        let state = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 2);
        assert!(state.contains(&KeyboardState::Press(code)));
        assert!(state.contains(&KeyboardState::Active(code)));

        // Act
        let state = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 1);
        assert!(state.contains(&KeyboardState::Active(code)));

        // Act
        tracker.register_event(release);
        let state = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 1);
        assert!(state.contains(&KeyboardState::Release(code)));
    }

    #[test]
    fn test_case_3() {
        // Assign
        let mut tracker = KeyboardTracker::new();
        let code = KeyCode::Char('b');
        let press = KeyEvent::new_with_kind(
            KeyCode::Char('b'),
            KeyModifiers::empty(),
            KeyEventKind::Press,
        );
        let release = KeyEvent::new_with_kind(
            KeyCode::Char('b'),
            KeyModifiers::empty(),
            KeyEventKind::Release,
        );

        // Act
        tracker.register_event(press);
        tracker.register_event(release);
        let state = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 3);
        assert!(state.contains(&KeyboardState::Press(code)));
        assert!(state.contains(&KeyboardState::Active(code)));
        assert!(state.contains(&KeyboardState::Release(code)));

        // Act
        let state = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 0);
    }

    #[test]
    fn test_case_4() {
        // Assign
        let mut tracker = KeyboardTracker::new();
        let code = KeyCode::Char('b');
        let press = KeyEvent::new_with_kind(
            KeyCode::Char('b'),
            KeyModifiers::empty(),
            KeyEventKind::Press,
        );
        let release = KeyEvent::new_with_kind(
            KeyCode::Char('b'),
            KeyModifiers::empty(),
            KeyEventKind::Release,
        );

        // Act
        tracker.register_event(press);
        let state = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 2);
        assert!(state.contains(&KeyboardState::Press(code)));
        assert!(state.contains(&KeyboardState::Active(code)));

        // Act
        tracker.register_event(release);
        tracker.register_event(press);
        let state = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 3);
        assert!(state.contains(&KeyboardState::Press(code)));
        assert!(state.contains(&KeyboardState::Active(code)));
        assert!(state.contains(&KeyboardState::Release(code)));

        // Act
        tracker.register_event(release);
        let state = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 1);
        assert!(state.contains(&KeyboardState::Release(code)));
    }

    #[test]
    fn test_case_5() {
        // Assign
        let mut tracker = KeyboardTracker::new();
        let code = KeyCode::Char('b');
        let press = KeyEvent::new_with_kind(
            KeyCode::Char('b'),
            KeyModifiers::empty(),
            KeyEventKind::Press,
        );
        let release = KeyEvent::new_with_kind(
            KeyCode::Char('b'),
            KeyModifiers::empty(),
            KeyEventKind::Release,
        );

        // Act
        tracker.register_event(press);
        let state = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 2);
        assert!(state.contains(&KeyboardState::Press(code)));
        assert!(state.contains(&KeyboardState::Active(code)));

        // Act
        tracker.register_event(press);
        tracker.register_event(release);
        let state = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 1);
        assert!(state.contains(&KeyboardState::Release(code)));

        // Act
        let state = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 0);
    }

    #[test]
    fn test_case_6() {
        // Assign
        let mut tracker = KeyboardTracker::new();
        let code = KeyCode::Char('b');
        let press = KeyEvent::new_with_kind(
            KeyCode::Char('b'),
            KeyModifiers::empty(),
            KeyEventKind::Press,
        );
        let release = KeyEvent::new_with_kind(
            KeyCode::Char('b'),
            KeyModifiers::empty(),
            KeyEventKind::Release,
        );

        // Act
        tracker.register_event(press);
        let state = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 2);
        assert!(state.contains(&KeyboardState::Press(code)));
        assert!(state.contains(&KeyboardState::Active(code)));

        // Act
        tracker.register_event(press);
        tracker.register_event(release);
        tracker.register_event(press);
        let state = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 3);
        assert!(state.contains(&KeyboardState::Press(code)));
        assert!(state.contains(&KeyboardState::Active(code)));
        assert!(state.contains(&KeyboardState::Release(code)));

        // Act
        let state = tracker.calculate_state();

        // Assert
        assert_eq!(state.len(), 1);
        assert!(state.contains(&KeyboardState::Active(code)));
    }
}
