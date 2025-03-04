use bevy::{math::I64Vec3, prelude::*};

use super::block::BlockState;

#[derive(Component)]
pub struct Generator();

pub type GeneratorFn<T> = fn(I64Vec3) -> T;
pub type BlockGenerator = GeneratorFn<BlockState>;

pub fn flat<const LEVEL: i64>(pos: I64Vec3) -> BlockState {
    pos.y < LEVEL
}

/// Debug block generators
/// Unlikely to be useful in real games, but useful for
/// testing and profiling rendering
pub mod debug {
    use std::f32;

    use bevy::math::{ops::sin, I64Vec3};

    use crate::world::block::BlockState;

    pub fn sine<
        const LEVEL: i64,
        const AMPLITUDE: i32,
        const PERIOD: i32>
    (pos: I64Vec3)
    -> BlockState {
        pos.y < (sin(pos.x as f32 * 2.0 * f32::consts::PI / PERIOD as f32) * (AMPLITUDE as f32)) as i64 + LEVEL
    }

    pub fn empty(_: I64Vec3) -> BlockState {
        false
    }

    pub fn full(_: I64Vec3) -> BlockState {
        true
    }
}
