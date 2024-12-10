//! Demonstrates how to enqueue custom draw commands in a render phase.
//!
//! This example shows how to use the built-in
//! [`bevy_render::render_phase::BinnedRenderPhase`] functionality with a
//! custom [`RenderCommand`] to allow inserting arbitrary GPU drawing logic
//! into Bevy's pipeline. This is not the only way to add custom rendering code
//! into Bevy—render nodes are another, lower-level method—but it does allow
//! for better reuse of parts of Bevy's built-in mesh rendering logic.

use bevy::{
    math::Vec3A,
    prelude::*,
    render::primitives::Aabb,
};
use vkxl::render::{VoxelRendererPlugin, PulledCube};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(VoxelRendererPlugin)
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



