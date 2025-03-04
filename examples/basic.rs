mod shared;

use bevy::{math::{I64Vec3, Vec3A}, pbr::CascadeShadowConfigBuilder, prelude::*, render::primitives::Aabb};
use shared::SharedUtilitiesPlugin;
use vkxl::{world::{chunk::Chunk16, generation, Level, Load}, VoxelPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, VoxelPlugin, SharedUtilitiesPlugin))
        .add_systems(Startup, setup)
        .add_systems(Startup, create_shape);
    // Make sure to tell Bevy to check our entity for visibility. Bevy won't
    // do this by default, for efficiency reasons.

    // We make sure to add these to the render app, not the main app.

    app.run();
}

/// Spawns the objects in the scene.
fn setup(mut commands: Commands) {
    let world = Level {
        generator: generation::debug::sine::<8, 3, 10>,
    };

    let id = commands.spawn((
        Visibility::default(),
        Transform::from_translation(Vec3::new(-1.5, 0.1, 0.0)),
        // This `Aabb` is necessary for the visibility checks to work.
        Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::splat(10.0),
        },
        world,
    )).id();

    // let chunk = Chunk16::generate(generation::debug::sine::<5, 3, 10>, I64Vec3::ZERO);

    //     commands.spawn((
    //         Visibility::default(),
    //         Transform::from_translation(Vec3::new(-1.5, 0.1, 0.0)),
    //         // This `Aabb` is necessary for the visibility checks to work.
    //         Aabb {
    //             center: Vec3A::ZERO,
    //             half_extents: Vec3A::splat(10.0),
    //         },
    //         chunk,
    //     ));

    let world2 = Level {
        generator: generation::debug::sine::<8, 3, 10>,
    };

    let mut level = (id, &world2);

    for x in 0..16 {
        for z in 0..16 {
            level.load::<16>(I64Vec3::new(x, 0, z), &mut commands);
        }
    }



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
            first_cascade_far_bound: 1.0,
            maximum_distance: 100.0,
            minimum_distance: 0.1,
            // num_cascades: 100,
            ..default()
        }
        .build(),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::BLACK,
        brightness: 0.00,
    });
}

fn create_shape(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::default());
    let material = materials.add(StandardMaterial {
        base_color: Color::Srgba(Srgba::new(0.2, 0.7, 0.3, 1.0)),
        ..Default::default()
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(-0.4, 0.3, -0.2),
    ));
}
