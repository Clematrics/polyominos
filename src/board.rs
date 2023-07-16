use std::ops::{Deref, DerefMut};

use crate::rotation::Rotation;

pub const SIZE: usize = 32;

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Board(pub [[bool; SIZE]; SIZE]);

impl Deref for Board {
    type Target = [[bool; SIZE]; SIZE];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Board {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Board {
    pub fn new(value: bool) -> Self {
        Board([[value; SIZE]; SIZE])
    }

    pub fn sub(&mut self, other: &Self) {
        for i in 0..SIZE {
            for j in 0..SIZE {
                if other[i][j] {
                    self[i][j] = false;
                }
            }
        }
    }
}

pub fn rotate_board(b: &Board, dim: (u8, u8), r: &Rotation) -> Board {
    let mut arr = [[false; SIZE]; SIZE];

    match r {
        Rotation::R0 => return b.clone(),
        Rotation::R90 => {
            let dim = (dim.1 as usize, dim.0 as usize);
            for i in 0..dim.0 {
                for j in 0..dim.1 {
                    arr[i][j] = b[j][dim.0 - i - 1];
                }
            }
        }
        Rotation::R180 => {
            let dim = (dim.0 as usize, dim.1 as usize);
            for i in 0..dim.0 {
                for j in 0..dim.1 {
                    arr[i][j] = b[dim.0 - i - 1][dim.1 - j - 1];
                }
            }
        }
        Rotation::R270 => {
            let dim = (dim.1 as usize, dim.0 as usize);
            for i in 0..dim.0 {
                for j in 0..dim.1 {
                    arr[i][j] = b[dim.1 - j - 1][i];
                }
            }
        }
    }

    Board(arr)
}
