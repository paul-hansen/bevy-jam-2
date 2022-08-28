use bevy::ecs::system::EntityCommands;

pub mod brain_dead;
pub mod coward;
pub mod hunter;
pub mod speedy;

#[derive(Debug, Copy, Clone)]
pub enum Bot {
    BrainDead,
    Speedy,
    Coward,
    Hunter,
}

impl Bot {
    pub fn insert(&self, commands: &mut EntityCommands) {
        match self {
            Bot::BrainDead => {
                commands.insert(brain_dead::BrainDead::default());
            }
            Bot::Speedy => {
                commands.insert(speedy::Speedy::default());
            }
            Bot::Coward => {
                commands.insert(coward::Coward::default());
            }
            Bot::Hunter => {
                commands.insert(hunter::Hunter::default());
            }
        }
    }
}
