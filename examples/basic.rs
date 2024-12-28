mod shared;

use bevy::{math::Vec3A, pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap}, prelude::*, render::primitives::Aabb};
use shared::SharedUtilitiesPlugin;
use vkxl::render::{PulledCube, VoxelRendererPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, VoxelRendererPlugin, SharedUtilitiesPlugin))
        .add_systems(Startup, setup)
        .insert_resource(DirectionalLightShadowMap { size: 4096 });
    // Make sure to tell Bevy to check our entity for visibility. Bevy won't
    // do this by default, for efficiency reasons.

    // We make sure to add these to the render app, not the main app.

    app.run();
}

/// Spawns the objects in the scene.
fn setup(
    mut commands: Commands,
) {
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
        Transform::from_translation(Vec3::new(-1.5, 0.2, 0.0)),
        // This `Aabb` is necessary for the visibility checks to work.
        Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::splat(0.5),
        },
        PulledCube,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform::default().looking_to(Vec3::new(-1., -1.5, -0.5), Vec3::Y),
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
        .build(),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::BLACK,
        brightness: 0.00,
    });
}
