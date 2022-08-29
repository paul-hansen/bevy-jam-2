use crate::{BoidColor, Bot, PlayerActions};
use bevy::prelude::*;
use leafwing_input_manager::buttonlike::MouseMotionDirection;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::user_input::InputKind;
use std::fmt::Formatter;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum PlayerType {
    #[default]
    AnyDevice,
    Wasd,
    ArrowKeys,
    Mouse,
    GamePad(Option<Gamepad>),
    Bot(Bot),
}

impl PlayerType {
    pub fn is_local(&self) -> bool {
        !matches!(self, Self::Bot(_))
    }

    pub fn human_options() -> [Self; 9] {
        [
            Self::AnyDevice,
            Self::Wasd,
            Self::Mouse,
            Self::ArrowKeys,
            Self::GamePad(None),
            Self::GamePad(Some(Gamepad { id: 0 })),
            Self::GamePad(Some(Gamepad { id: 1 })),
            Self::GamePad(Some(Gamepad { id: 2 })),
            Self::GamePad(Some(Gamepad { id: 3 })),
        ]
    }

    pub fn bot_options() -> [Self; 4] {
        [
            Self::Bot(Bot::BoneHead),
            Self::Bot(Bot::ScaredyCat),
            Self::Bot(Bot::Speedy),
            Self::Bot(Bot::Hunter),
        ]
    }

    pub fn human_bot_label(&self) -> &str {
        match self.is_local() {
            true => "Human",
            false => "Bot",
        }
    }

    pub fn input_map(&self) -> Option<InputMap<PlayerActions>> {
        match self {
            PlayerType::AnyDevice => Some(
                PlayerType::Wasd
                    .input_map()
                    .unwrap()
                    .merge(&PlayerType::ArrowKeys.input_map().unwrap())
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
            PlayerType::ArrowKeys => Some(
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
            PlayerType::GamePad(gp) => Some({
                let mut map = InputMap::<PlayerActions>::default()
                    .insert(DualAxis::left_stick(), PlayerActions::Direction)
                    .insert(VirtualDPad::dpad(), PlayerActions::CameraZoom)
                    .insert(GamepadButtonType::South, PlayerActions::Boost)
                    .insert(GamepadButtonType::RightTrigger, PlayerActions::Boost)
                    .build();
                if let Some(gp) = gp {
                    map.set_gamepad(*gp);
                }
                map
            }),
            PlayerType::Bot(_) => None,
        }
    }
}

impl std::fmt::Display for PlayerType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerType::AnyDevice => write!(f, "All Devices"),
            PlayerType::GamePad(Some(gamepad)) => write!(f, "Gamepad {}", gamepad.id + 1),
            PlayerType::GamePad(None) => write!(f, "Any Gamepad"),
            PlayerType::Bot(b) => write!(f, "{}", b),
            _ => write!(f, "{self:?}"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PlayerSettings {
    pub player_type: PlayerType,
    pub color: BoidColor,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub enum MultiplayerMode {
    #[default]
    SplitScreenVertical,
    SplitScreenHorizontal,
    SharedScreen,
}

impl std::fmt::Display for MultiplayerMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MultiplayerMode::SharedScreen => write!(f, "Shared Screen"),
            MultiplayerMode::SplitScreenVertical => write!(f, "Split-screen Prefer Vertical"),
            MultiplayerMode::SplitScreenHorizontal => write!(f, "Split-screen Prefer Horizontal"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoundSettings {
    pub players: Vec<PlayerSettings>,
    pub arena_radius: f32,
    pub boid_count: f32,
    pub multiplayer_mode: MultiplayerMode,
}

impl RoundSettings {
    pub fn local_player_count(&self) -> usize {
        self.players
            .iter()
            .filter(|p| p.player_type.is_local())
            .count()
    }

    // Gets the index of the viewport this player was assigned based on how many local players
    // came before this player.
    pub fn player_viewport_id(&self, player_index: usize) -> Option<usize> {
        self.players
            .iter()
            .enumerate()
            .filter(|(_, p)| p.player_type.is_local())
            .enumerate()
            .find(|(_, (i, _))| *i == player_index)
            .map(|(i, (_, _))| i)
    }
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
                    player_type: PlayerType::Bot(Bot::BoneHead),
                    color: BoidColor::Green,
                },
                PlayerSettings {
                    player_type: PlayerType::Bot(Bot::BoneHead),
                    color: BoidColor::Blue,
                },
                PlayerSettings {
                    player_type: PlayerType::Bot(Bot::ScaredyCat),
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
                    player_type: PlayerType::Bot(Bot::BoneHead),
                    color: BoidColor::Pink,
                },
                PlayerSettings {
                    player_type: PlayerType::Bot(Bot::BoneHead),
                    color: BoidColor::Cyan,
                },
            ],
            arena_radius: 1200.0,
            boid_count: 400.0,
            multiplayer_mode: MultiplayerMode::default(),
        }
    }
}
