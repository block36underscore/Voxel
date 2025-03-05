use std::f32::consts::PI;

use bevy::{
    app::{App, Plugin},
    prelude::*,
    reflect::Reflect,
    window::{CursorGrabMode, PrimaryWindow},
};

pub struct SharedUtilitiesPlugin;

impl Plugin for SharedUtilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (cursor_grab, spawn_player, spawn_perf_ui))
            .add_systems(Update, handle_player_input)
            .add_plugins((
                InputManagerPlugin::<Action>::default(),
                PerfUiPlugin,
                bevy::diagnostic::FrameTimeDiagnosticsPlugin,
                bevy::diagnostic::EntityCountDiagnosticsPlugin,
                bevy::diagnostic::SystemInformationDiagnosticsPlugin,
                bevy::render::diagnostic::RenderDiagnosticsPlugin,
            ));
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum Action {
    MoveLeft,
    MoveRight,
    MoveForeward,
    MoveBackward,
    MoveDown,
    MoveUp,
    Pan,
    LeftClick,
    RightClick,
    Escape,
}

impl Actionlike for Action {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            Pan => InputControlKind::DualAxis,
            _ => InputControlKind::Button,
        }
    }
}

use iyes_perf_ui::{prelude::PerfUiAllEntries, PerfUiPlugin};
use leafwing_input_manager::{
    plugin::InputManagerPlugin,
    prelude::{ActionState, InputMap, MouseMove},
    Actionlike, InputControlKind, InputManagerBundle,
};
use Action::*;

impl Action {
    pub const DIRECTION_CONTROLS: [Action; 6] = [
        MoveLeft,
        MoveRight,
        MoveForeward,
        MoveBackward,
        MoveDown,
        MoveUp,
    ];

    pub fn get_direction(&self) -> Option<Vec3> {
        match self {
            MoveLeft => Some(Vec3::new(-1.0, 0.0, 0.0)),
            MoveRight => Some(Vec3::new(1.0, 0.0, 0.0)),
            MoveForeward => Some(Vec3::new(0.0, 0.0, -1.0)),
            MoveBackward => Some(Vec3::new(0.0, 0.0, 1.0)),
            MoveDown => Some(Vec3::new(0.0, -1.0, 0.0)),
            MoveUp => Some(Vec3::new(0.0, 1.0, 0.0)),
            _ => None,
        }
    }
}

pub fn setup_controls() -> InputMap<Action> {
    let mut map = InputMap::default();
    map.insert(MoveLeft, KeyCode::KeyA)
        .insert(MoveRight, KeyCode::KeyD)
        .insert(MoveForeward, KeyCode::KeyW)
        .insert(MoveBackward, KeyCode::KeyS)
        .insert(MoveDown, KeyCode::ShiftLeft)
        .insert(MoveUp, KeyCode::Space)
        .insert(Action::Escape, KeyCode::Escape)
        .insert(LeftClick, MouseButton::Left)
        .insert(RightClick, MouseButton::Right)
        .insert_dual_axis(Pan, MouseMove::default());

    map
}

pub fn cursor_grab(mut q_windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = q_windows.single_mut();
    primary_window.cursor_options.grab_mode = CursorGrabMode::Locked;
    primary_window.cursor_options.visible = false;
}

#[derive(Component)]
#[require(ViewVisibility, Transform)]
pub struct Player;

#[derive(Component)]
#[require(Transform)]
pub struct Camera;

pub fn spawn_player(mut commands: Commands) {
    let mut look_transform = Transform::from_xyz(5.0, 5.0, 5.0);
    look_transform.look_at(Vec3::ZERO, Vec3::Y);
    look_transform.translation = Vec3::ZERO;
    let camera = commands
        .spawn((Camera, Camera3d::default(), look_transform))
        .id();
    let mut player = commands.spawn((
        Transform::from_xyz(5.0, 5.0, 5.0),
        Player,
        InputManagerBundle::with_map(setup_controls()),
    ));

    player.add_child(camera);
}

pub const SPEED: f32 = 20.0;
pub const LOOK_SPEED: f32 = 0.00075;

pub fn handle_player_input(
    mut player: Query<(&ActionState<Action>, &mut Transform), With<Player>>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    let (action_state, mut player_transform) = player.single_mut();
    let mut camera_transform = camera.single_mut();
    let (mut yaw, mut pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);

    // Movement Inputs

    let mut velocity = Vec3::ZERO;

    for action in Action::DIRECTION_CONTROLS {
        if action_state.pressed(&action) {
            if let Some(direction) = action.get_direction() {
                velocity += Quat::from_axis_angle(Vec3::Y, yaw) * direction;
            }
        }
    }

    velocity = velocity.normalize_or_zero();
    player_transform.translation += velocity * time.delta().as_secs_f32() * SPEED;

    // Camera Inputs

    let window = &mut q_windows.single_mut();
    let cursor = &mut window.cursor_options;
    let camera_delta = action_state.axis_pair(&Action::Pan);

    if cursor.grab_mode != CursorGrabMode::None {
        yaw -= camera_delta.x * LOOK_SPEED;
        pitch -= camera_delta.y * LOOK_SPEED;
        pitch = pitch.clamp(-PI / 2.0, PI / 2.0);

        camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    }

    // Window Focus Inputs

    if cursor.grab_mode != CursorGrabMode::None && action_state.just_pressed(&Action::Escape) {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
        let cursor_pos = Some(window.size() / 2.0);
        window.set_cursor_position(cursor_pos);
    } else if cursor.grab_mode == CursorGrabMode::None
        && (action_state.just_pressed(&Action::LeftClick)
            || action_state.just_pressed(&Action::RightClick))
    {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
}

fn spawn_perf_ui(mut commands: Commands) {
    commands.spawn(PerfUiAllEntries::default());
}
