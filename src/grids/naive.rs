use std::{
    fmt::{Debug, Write},
    ops::BitOrAssign,
};

use crate::{grid::Grid, rotation::Rotation};

const SIZE: usize = 32;

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Naive([[bool; SIZE]; SIZE]);

impl Grid for Naive {
    fn new() -> Self {
        Self([[false; SIZE]; SIZE])
    }

    fn get(&self, x: usize, y: usize) -> bool {
        self.0[x][y]
    }

    fn set(&mut self, x: usize, y: usize) {
        self.0[x][y] = true;
    }

    fn unset(&mut self, x: usize, y: usize) {
        self.0[x][y] = false;
    }

    fn count(&self) -> u32 {
        self.0
            .iter()
            .map(|coln| coln.iter().map(|e| *e as u32).sum::<u32>())
            .sum()
    }

    fn reserve_space(&mut self, x: usize, y: usize) {
        assert!(x < SIZE && y < SIZE);
    }

    fn get_bounding_box(&self) -> (usize, usize) {
        let mut dim_x = 0;
        let mut dim_y = 0;

        for x in (0..SIZE).rev() {
            if self.0[x].contains(&true) {
                dim_x = x + 1;
                break;
            }
        }

        for x in 0..dim_x {
            for y in (0..SIZE).rev() {
                if self.0[x][y] == true {
                    dim_y = dim_y.max(y + 1);
                    break;
                }
            }
        }

        (dim_x, dim_y)
    }

    fn shift_x(&mut self, n: isize) {
        use std::cmp::Ordering;

        match n.cmp(&0) {
            Ordering::Equal => (),
            Ordering::Greater => {
                self.0.copy_within(0..SIZE - n as usize, n as usize);
                self.0[0..n as usize].fill([false; SIZE]);
            }
            Ordering::Less => {
                self.0.copy_within((-n) as usize..SIZE, 0);
                self.0[SIZE - (-n) as usize..SIZE].fill([false; SIZE]);
            }
        }
    }

    fn shift_y(&mut self, n: isize) {
        use std::cmp::Ordering;

        match n.cmp(&0) {
            Ordering::Equal => (),
            Ordering::Greater => {
                for coln in self.0.iter_mut() {
                    coln.copy_within(0..SIZE - n as usize, n as usize);
                    coln[0..n as usize].fill(false);
                }
            }
            Ordering::Less => {
                for coln in self.0.iter_mut() {
                    coln.copy_within((-n) as usize..SIZE, 0);
                    coln[SIZE - (-n) as usize..SIZE].fill(false);
                }
            }
        }
    }

    fn rotate(&self, dim: (u8, u8), r: Rotation) -> Self {
        let mut new = Self::new();
        let dim = (dim.0 as usize, dim.1 as usize);

        match r {
            Rotation::R0 => {
                for x in 0..dim.0 {
                    for y in 0..dim.1 {
                        new.0[x][y] = self.0[x][y];
                    }
                }
            }
            Rotation::R90 => {
                // x and y are in the frame of reference
                // of the new grid, with dimensions reversed,
                // i.e. (dim.1, dim.0)
                for x in 0..dim.1 {
                    for y in 0..dim.0 {
                        new.0[x][y] = self.0[y][dim.1 - 1 - x];
                    }
                }
            }
            Rotation::R180 => {
                for x in 0..dim.0 {
                    for y in 0..dim.1 {
                        new.0[x][y] = self.0[dim.0 - 1 - x][dim.1 - 1 - y];
                    }
                }
            }
            Rotation::R270 => {
                for x in 0..dim.1 {
                    for y in 0..dim.0 {
                        new.0[x][y] = self.0[dim.0 - 1 - y][x];
                    }
                }
            }
        }

        new
    }
}

impl BitOrAssign for Naive {
    fn bitor_assign(&mut self, rhs: Self) {
        for x in 0..SIZE {
            for y in 0..SIZE {
                self.0[x][y] |= rhs.0[x][y];
            }
        }
    }
}

impl Debug for Naive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dim = self.get_bounding_box();
        for y in (0..dim.1).rev() {
            for x in 0..dim.0 {
                if self.0[x][y] {
                    f.write_char('O')?;
                } else {
                    f.write_char('.')?;
                }
            }
            if y != 0 {
                f.write_char('\n')?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naive_dim() {
        assert_eq!(Naive::new().get_bounding_box(), (0, 0));

        let mut grid = Naive::new();
        grid.set(1, 2);
        assert_eq!(grid.get_bounding_box(), (2, 3));

        grid.set(2, 1);
        assert_eq!(grid.get_bounding_box(), (3, 3));

        grid.unset(1, 2);
        assert_eq!(grid.get_bounding_box(), (3, 2));
    }
}
