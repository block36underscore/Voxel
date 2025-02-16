use crate::render::buffers::{Cube, ToCubes};

pub type BlockState = bool;

impl ToCubes for bool {
    fn to_cubes(&self) -> Vec<Cube> {
        if *self {
            vec![Cube::default()]
        } else {
            vec![]
        }
    }
}
