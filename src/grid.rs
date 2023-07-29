use std::{fmt::Debug, hash::Hash};

use crate::rotation::Rotation;

/// A representation of the ℕxℕ grid, which maps coordinates to a boolean
/// There are a finite number of elements set to true, thus there exists
/// a finite bounding box surrounding all true elements.
/// Rotating the grid means rotating only the elements inside the bounding
/// box of the given dimension, such that the new corner (0, 0) is mapped
/// by a corner of the bounding box:
///
/// ```text
/// ↑        ↑
/// ├───┐    │
/// │x  │    ├──────┐
/// │   │    │      │
/// │   │    │x     │
/// └───┴→   └──────┴→
/// ```
pub trait Grid: Clone + Debug + Ord + Hash {
    /// Create a new grid with all elements set to false
    fn new() -> Self;

    /// Reserve space to ensure that the grid can contain
    /// at least all elements in (0, 0) -> (x, y) **inclusive**
    fn reserve_space(&mut self, x: usize, y: usize);

    /// Returns the size of the zone between (0, 0) -> (x, y) **exclusive**
    /// such that all elements are inside. This returns (x, y)
    fn get_bounding_box(&self) -> (usize, usize);

    /// Set the grid element of coordinates (x, y) to true
    fn set(&mut self, x: usize, y: usize);
    /// Set the grid element of coordinates (x, y) to false
    fn unset(&mut self, x: usize, y: usize);
    /// Return the value of the grid element at coordinates (x, y)
    fn get(&self, x: usize, y: usize) -> bool;

    /// Return the number of elements mapped to true
    fn count(&self) -> u32;

    /// Shift the grid in the X direction. New elements are set to false
    fn shift_x(&mut self, n: isize);
    /// Shift the grid in the Y direction. New elements are set to false
    fn shift_y(&mut self, n: isize);

    /// Rotate the portion of the grid of dimension dim
    fn rotate(&self, dim: (u8, u8), r: Rotation) -> Self;
}

pub fn transfer<T, U>(from: &T) -> U
where
    T: Grid,
    U: Grid,
{
    let mut to = U::new();
    let dim = from.get_bounding_box();
    for x in 0..dim.0 {
        for y in 0..dim.1 {
            if from.get(x, y) {
                to.set(x, y);
            }
        }
    }
    to
}

pub fn are_equal<T, U>(lhs: &T, rhs: &U)
where
    T: Grid,
    U: Grid,
{
    let bb_lhs = lhs.get_bounding_box();
    let bb_rhs = rhs.get_bounding_box();
    assert_eq!(
        bb_lhs, bb_rhs,
        "Bounding boxes are not equal:\nlhs:\n{:?}\nrhs:\n{:?}",
        lhs, rhs
    );

    for x in 0..bb_lhs.0 {
        for y in 0..bb_lhs.1 {
            assert_eq!(
                lhs.get(x, y),
                rhs.get(x, y),
                "lhs and rhs differ:\nlhs:\n{:?}\nrhs:\n{:?}",
                lhs,
                rhs
            );
        }
    }
}
