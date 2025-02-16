use bevy::app::Plugin;
use render::VoxelRendererPlugin;

pub mod render;
pub mod world;

pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(VoxelRendererPlugin);
    }
}
