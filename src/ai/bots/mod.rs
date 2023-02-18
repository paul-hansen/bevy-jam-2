use bevy::ecs::system::EntityCommands;
use bevy::prelude::{FromReflect, Reflect};
use std::fmt::Formatter;

pub mod bonehead;
pub mod coward;
pub mod hunter;
pub mod speedy;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Reflect, FromReflect)]
pub enum Bot {
    #[default]
    BoneHead,
    Speedy,
    ScaredyCat,
    Hunter,
}

impl std::fmt::Display for Bot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Bot::BoneHead => write!(f, "{}", bonehead::BoneHead::default()),
            Bot::Speedy => write!(f, "{}", speedy::Speedy::default()),
            Bot::ScaredyCat => write!(f, "{}", coward::ScaredyCat::default()),
            Bot::Hunter => write!(f, "{}", hunter::Hunter::default()),
        }
    }
}

impl Bot {
    pub fn insert(&self, commands: &mut EntityCommands) {
        match self {
            Bot::BoneHead => {
                commands.insert(bonehead::BoneHead::default());
            }
            Bot::Speedy => {
                commands.insert(speedy::Speedy::default());
            }
            Bot::ScaredyCat => {
                commands.insert(coward::ScaredyCat::default());
            }
            Bot::Hunter => {
                commands.insert(hunter::Hunter::default());
            }
        }
    }
}
