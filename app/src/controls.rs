use chargrid::input::{Input, KeyboardInput};
use direction::CardinalDirection;
use maplit::btreemap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppInput {
    Direction(CardinalDirection),
    Wait,
    MessageLog,
    Get,
    DropItem,
    ApplyItem,
    UnequipWeapon,
    RemoveArmour,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppInputDriving {
    ContinueDriving,
    StopDriving,
}

#[derive(Serialize, Deserialize)]
pub struct Controls {
    walking_keys: BTreeMap<KeyboardInput, AppInput>,
    driving_keys: BTreeMap<KeyboardInput, AppInputDriving>,
}

impl Default for Controls {
    fn default() -> Self {
        let walking_keys = btreemap![
            KeyboardInput::Left => AppInput::Direction(CardinalDirection::West),
            KeyboardInput::Right => AppInput::Direction(CardinalDirection::East),
            KeyboardInput::Up => AppInput::Direction(CardinalDirection::North),
            KeyboardInput::Down => AppInput::Direction(CardinalDirection::South),
            KeyboardInput::Char(' ') => AppInput::Wait,
            KeyboardInput::Char('m') => AppInput::MessageLog,
            KeyboardInput::Char('g') => AppInput::Get,
            KeyboardInput::Char('d') => AppInput::DropItem,
            KeyboardInput::Char('a') => AppInput::ApplyItem,
            KeyboardInput::Char('i') => AppInput::ApplyItem,
            KeyboardInput::Char('u') => AppInput::UnequipWeapon,
            KeyboardInput::Char('r') => AppInput::RemoveArmour,
        ];
        let driving_keys = btreemap![
            KeyboardInput::Char('d') => AppInputDriving::ContinueDriving,
            KeyboardInput::Char('s') => AppInputDriving::StopDriving
        ];
        Self {
            walking_keys,
            driving_keys,
        }
    }
}
impl Controls {
    pub fn get_walking(&self, input: Input) -> Option<AppInput> {
        match input {
            Input::Keyboard(keyboard_input) => self.walking_keys.get(&keyboard_input).cloned(),
            Input::Mouse(_) => None,
        }
    }

    pub fn get_driving(&self, input: Input) -> Option<AppInputDriving> {
        match input {
            Input::Keyboard(keyboard_input) => self.driving_keys.get(&keyboard_input).cloned(),
            Input::Mouse(_) => None,
        }
    }
}
