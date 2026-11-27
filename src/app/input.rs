use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Input {
    System(SystemInput),
    Game(GameInput),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum GameInput {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Cancel,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum SystemInput {
    Quit,
}

impl Input {
    pub(super) fn from_key_event(key: KeyEvent) -> Option<Input> {
        match key.code {
            KeyCode::Char('q' | 'Q') => Some(Input::System(SystemInput::Quit)),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Input::System(SystemInput::Quit))
            }
            KeyCode::Up => Some(Input::Game(GameInput::Up)),
            KeyCode::Down => Some(Input::Game(GameInput::Down)),
            KeyCode::Left => Some(Input::Game(GameInput::Left)),
            KeyCode::Right => Some(Input::Game(GameInput::Right)),
            KeyCode::Enter | KeyCode::Char('z' | 'Z') => Some(Input::Game(GameInput::Confirm)),
            KeyCode::Esc | KeyCode::Char('x' | 'X') => Some(Input::Game(GameInput::Cancel)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn q_keys_map_to_quit() {
        for key in [KeyCode::Char('q'), KeyCode::Char('Q')] {
            let input = Input::from_key_event(KeyEvent::from(key));
            assert_eq!(input, Some(Input::System(SystemInput::Quit)), "{key:?}");
        }
    }

    #[test]
    fn ctrl_c_maps_to_quit() {
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(
            Input::from_key_event(key),
            Some(Input::System(SystemInput::Quit))
        );
    }

    #[test]
    fn c_without_ctrl_maps_to_none() {
        let key = KeyEvent::from(KeyCode::Char('c'));
        assert_eq!(Input::from_key_event(key), None);
    }

    #[test]
    fn arrow_keys_map_to_direction() {
        for (key, expect) in [
            (KeyCode::Up, GameInput::Up),
            (KeyCode::Down, GameInput::Down),
            (KeyCode::Left, GameInput::Left),
            (KeyCode::Right, GameInput::Right),
        ] {
            let input = Input::from_key_event(KeyEvent::from(key));
            assert_eq!(input, Some(Input::Game(expect)), "{key:?}");
        }
    }

    #[test]
    fn confirm_keys_map_to_confirm() {
        for key in [KeyCode::Enter, KeyCode::Char('z'), KeyCode::Char('Z')] {
            let input = Input::from_key_event(KeyEvent::from(key));
            assert_eq!(input, Some(Input::Game(GameInput::Confirm)), "{key:?}");
        }
    }

    #[test]
    fn cancel_keys_map_to_cancel() {
        for key in [KeyCode::Esc, KeyCode::Char('x'), KeyCode::Char('X')] {
            let input = Input::from_key_event(KeyEvent::from(key));
            assert_eq!(input, Some(Input::Game(GameInput::Cancel)), "{key:?}");
        }
    }
}
