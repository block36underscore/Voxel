mod shared;

use bevy::{math::Vec3A, prelude::*, render::primitives::Aabb};
use shared::SharedUtilitiesPlugin;
use vkxl::render::{PulledCube, VoxelRendererPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, VoxelRendererPlugin, SharedUtilitiesPlugin))
        .add_systems(Startup, setup);
        // Make sure to tell Bevy to check our entity for visibility. Bevy won't
        // do this by default, for efficiency reasons.


    // We make sure to add these to the render app, not the main app.

    app.run();
}



/// Spawns the objects in the scene.
fn setup(mut commands: Commands) {
    // Spawn a single entity that has custom rendering. It'll be extracted into
    // the render world via [`ExtractComponent`].
    commands.spawn((
        Visibility::default(),
        Transform::from_translation(Vec3::new(0.1, 0.2, 0.0)),
        // This `Aabb` is necessary for the visibility checks to work.
        Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::splat(0.5),
        },
        PulledCube,
    ));

    commands.spawn((
        Visibility::default(),
        Transform::from_translation(Vec3::new(-0.5, 0.2, 0.0)),
        // This `Aabb` is necessary for the visibility checks to work.
        Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::splat(0.5),
        },
        PulledCube,
    ));

    // Spawn the camera.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
