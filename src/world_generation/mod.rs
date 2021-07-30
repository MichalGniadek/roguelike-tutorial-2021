mod cellular_automata;

use bevy::prelude::*;

pub struct WorldGenerationPlugins;
impl PluginGroup for WorldGenerationPlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        group.add(cellular_automata::CellularAutomataPlugin);
    }
}
