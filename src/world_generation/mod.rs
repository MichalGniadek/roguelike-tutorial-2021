mod cellular_automata;
mod drunkard_generation;

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorldGeneratorType {
    Drunkard,
    CellularAutomata,
}

pub struct WorldGenerationPlugins;
impl PluginGroup for WorldGenerationPlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        group
            .add(drunkard_generation::DrunkardPlugin)
            .add(cellular_automata::CellularAutomataPlugin);
    }
}
