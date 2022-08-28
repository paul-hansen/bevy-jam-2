use crate::{BoidColor, Bot, PlayerActions};
use bevy::prelude::*;
use leafwing_input_manager::buttonlike::MouseMotionDirection;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::user_input::InputKind;

#[derive(Debug, Copy, Clone)]
pub enum PlayerType {
    AnyDevice,
    Wasd,
    Arrowkeys,
    Mouse,
    GamePad(Option<Gamepad>),
    Bot(Bot),
}

impl PlayerType {
    pub fn is_local(&self) -> bool {
        !matches!(self, Self::Bot(_))
    }

    pub fn input_map(&self) -> Option<InputMap<PlayerActions>> {
        match self {
            PlayerType::AnyDevice => Some(
                PlayerType::Wasd
                    .input_map()
                    .unwrap()
                    .merge(&PlayerType::Arrowkeys.input_map().unwrap())
                    .merge(&PlayerType::Mouse.input_map().unwrap())
                    .merge(&PlayerType::GamePad(None).input_map().unwrap())
                    .build(),
            ),
            PlayerType::Wasd => Some(
                InputMap::<PlayerActions>::default()
                    .insert(VirtualDPad::wasd(), PlayerActions::Direction)
                    .insert(
                        VirtualDPad {
                            up: KeyCode::Equals.into(),
                            down: KeyCode::Minus.into(),
                            left: KeyCode::Minus.into(),
                            right: KeyCode::Equals.into(),
                        },
                        PlayerActions::CameraZoom,
                    )
                    .insert(
                        VirtualDPad {
                            up: KeyCode::R.into(),
                            down: KeyCode::F.into(),
                            left: KeyCode::R.into(),
                            right: KeyCode::F.into(),
                        },
                        PlayerActions::CameraZoom,
                    )
                    .insert(KeyCode::Space, PlayerActions::Boost)
                    .insert(KeyCode::LShift, PlayerActions::Boost)
                    .build(),
            ),
            PlayerType::Arrowkeys => Some(
                InputMap::<PlayerActions>::default()
                    .insert(VirtualDPad::arrow_keys(), PlayerActions::Rotate)
                    .insert(
                        VirtualDPad {
                            up: KeyCode::NumpadAdd.into(),
                            down: KeyCode::NumpadSubtract.into(),
                            left: KeyCode::NumpadSubtract.into(),
                            right: KeyCode::NumpadAdd.into(),
                        },
                        PlayerActions::CameraZoom,
                    )
                    .insert(KeyCode::Up, PlayerActions::Boost)
                    .build(),
            ),
            PlayerType::Mouse => Some(
                InputMap::<PlayerActions>::default()
                    .insert(
                        VirtualDPad {
                            up: InputKind::MouseMotion(MouseMotionDirection::Down),
                            down: InputKind::MouseMotion(MouseMotionDirection::Up),
                            left: InputKind::MouseMotion(MouseMotionDirection::Left),
                            right: InputKind::MouseMotion(MouseMotionDirection::Right),
                        },
                        PlayerActions::Direction,
                    )
                    .insert(VirtualDPad::mouse_wheel(), PlayerActions::CameraZoom)
                    .insert(MouseButton::Left, PlayerActions::Boost)
                    .build(),
            ),
            PlayerType::GamePad(_) => Some(
                InputMap::<PlayerActions>::default()
                    .insert(DualAxis::left_stick(), PlayerActions::Direction)
                    .insert(VirtualDPad::dpad(), PlayerActions::CameraZoom)
                    .insert(GamepadButtonType::South, PlayerActions::Boost)
                    .insert(GamepadButtonType::RightTrigger, PlayerActions::Boost)
                    .build(),
            ),
            PlayerType::Bot(_) => None,
        }
    }
}

pub struct PlayerSettings {
    pub player_type: PlayerType,
    pub color: BoidColor,
}

pub struct RoundSettings {
    pub players: Vec<PlayerSettings>,
}

impl Default for RoundSettings {
    fn default() -> Self {
        Self {
            players: vec![
                PlayerSettings {
                    player_type: PlayerType::AnyDevice,
                    color: BoidColor::Red,
                },
                PlayerSettings {
                    player_type: PlayerType::Bot(Bot::BrainDead),
                    color: BoidColor::Green,
                },
                PlayerSettings {
                    player_type: PlayerType::Bot(Bot::BrainDead),
                    color: BoidColor::Blue,
                },
                PlayerSettings {
                    player_type: PlayerType::Bot(Bot::Coward),
                    color: BoidColor::Yellow,
                },
                PlayerSettings {
                    player_type: PlayerType::Bot(Bot::Speedy),
                    color: BoidColor::Purple,
                },
                PlayerSettings {
                    player_type: PlayerType::Bot(Bot::Hunter),
                    color: BoidColor::Orange,
                },
                PlayerSettings {
                    player_type: PlayerType::Bot(Bot::BrainDead),
                    color: BoidColor::Pink,
                },
                PlayerSettings {
                    player_type: PlayerType::Bot(Bot::BrainDead),
                    color: BoidColor::Cyan,
                },
            ],
        }
    }
}
