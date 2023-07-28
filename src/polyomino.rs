use std::collections::hash_map::DefaultHasher;
use std::fmt::{self, Debug};
use std::hash::Hasher;

use crate::grid::{are_equal, transfer, Grid};
use crate::grids::naive::Naive;
use crate::rotation::Rotation;

#[derive(Copy, Clone)]
pub struct Polyomino<T>
where
    T: Grid,
{
    pub square_count: u8,
    pub dimension: (u8, u8),
    pub repr: T,
    pub mask: T,
}

impl<T> Debug for Polyomino<T>
where
    T: Grid,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Dimensions: {:?}\n", self.dimension))?;
        for j in (0..self.dimension.1 as usize).rev() {
            for i in 0..self.dimension.0 as usize {
                let str = {
                    if self.repr.get(i, j) {
                        if self.mask.get(i, j) {
                            "!"
                        } else {
                            "O"
                        }
                    } else {
                        if self.mask.get(i, j) {
                            "+"
                        } else {
                            "."
                        }
                    }
                };

                let to_print = if i < self.dimension.0 as usize && j < self.dimension.1 as usize {
                    format!("\x1b[32m{str}\x1b[m")
                } else {
                    format!("\x1b[31m{str}\x1b[m")
                };

                f.write_str(&to_print)?;
            }
            if j != 0 {
                f.write_str("\n")?;
            }
        }

        Ok(())
    }
}

impl<T> Polyomino<T>
where
    T: Grid,
{
    pub fn trivial() -> Self {
        let repr = {
            let mut board = T::new();
            board.reserve_space(3, 3);
            board.set(1, 1);
            board
        };
        let mask = {
            let mut board = T::new();
            board.reserve_space(3, 3);
            board.set(1, 0);
            board.set(0, 1);
            board.set(1, 2);
            board.set(2, 1);
            board
        };

        Self {
            square_count: 1,
            dimension: (3, 3),
            repr,
            mask,
        }
    }

    pub fn from(dimension: (u8, u8), repr: T, mask: T) -> Self {
        Self {
            square_count: repr.count() as u8,
            dimension,
            repr,
            mask,
        }
    }

    pub fn add_square(&mut self, mut x: usize, mut y: usize, anti_mask: &T) {
        self.square_count += 1;
        // The anti_mask is not shifted like the repr and mask boards, for efficiency reasons
        // Thus, we need offsets on coordinates to access the anti_mask
        let (mut x_offset, mut y_offset) = (0, 0);

        if x + 1 == self.dimension.0 as usize {
            self.dimension.0 += 1;
        }
        if y + 1 == self.dimension.1 as usize {
            self.dimension.1 += 1;
        }
        if x == 0 {
            x_offset = 1;
            self.dimension.0 += 1;
            x += 1;
        }
        if y == 0 {
            y_offset = 1;
            self.dimension.1 += 1;
            y += 1;
        }
        // NOTE: there is no need to optimise for a shift both in x and y,
        // since no square can be added here (otherwise it would not be connected)

        // Prepare the bounding box.
        self.repr
            .reserve_space(self.dimension.0 as usize, self.dimension.1 as usize);
        self.mask
            .reserve_space(self.dimension.0 as usize, self.dimension.1 as usize);

        // Apply offsets
        if x_offset == 1 {
            self.repr.shift_x(1);
            self.mask.shift_x(1);
        }
        if y_offset == 1 {
            self.repr.shift_y(1);
            self.mask.shift_y(1);
        }

        self.repr.set(x, y);
        self.mask.unset(x, y);
        for (x, y) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)].into_iter() {
            let anti_mask = if x == 0 || y == 0 {
                false
            } else {
                anti_mask.get(x - x_offset, y - y_offset)
            };
            if !self.repr.get(x, y) && !anti_mask {
                // println!("Set mask at {x}, {y}");
                self.mask.set(x, y)
            };
        }
    }
}

/// Return all polyominoes that can be created by adding a square to this polyomino, excluding positions out of the mask
pub fn decline<T>(p: &Polyomino<T>) -> Vec<Polyomino<T>>
where
    T: Grid,
{
    let mut polyominoes = vec![];
    // println!("From polyomino:");
    // println!("{p:?}");

    let mut hasher = DefaultHasher::new();
    p.repr.hash(&mut hasher);
    let hash = hasher.finish();
    println!("Hash is {:}", hash);

    let mut mask_witness = transfer::<_, Naive>(&p.mask);
    let mut anti_mask_witness = Naive::new();
    anti_mask_witness.reserve_space((p.dimension.0 + 1) as usize, (p.dimension.1 + 1) as usize);

    let mut mask = p.mask.clone();
    let mut anti_mask = T::new();
    anti_mask.reserve_space((p.dimension.0 + 1) as usize, (p.dimension.1 + 1) as usize);
    println!("Mask is\n{:?}\nWitness mask is\n{:?}", mask, mask_witness);
    for x in 0..p.dimension.0 as usize {
        for y in 0..p.dimension.1 as usize {
            if p.mask.get(x, y) {
                // add a new polyomino
                let mut new_p = Polyomino {
                    square_count: p.square_count,
                    dimension: p.dimension,
                    repr: p.repr.clone(),
                    mask: mask.clone(),
                };

                let mut witness = Polyomino::<Naive> {
                    square_count: p.square_count,
                    dimension: p.dimension,
                    repr: transfer(&p.repr),
                    mask: mask_witness.clone(),
                };

                println!("Add square at {x},{y}");
                new_p.add_square(x, y, &anti_mask);
                anti_mask.set(x, y);
                mask.unset(x, y);

                witness.add_square(x, y, &anti_mask_witness);
                anti_mask_witness.set(x, y);
                mask_witness.unset(x, y);

                println!(
                    "Now polyomino of dim {:?}:\n{:?}\nWith mask\n{:?}",
                    new_p.dimension, new_p.repr, new_p.mask
                );
                println!(
                    "Now witness of dim {:?}:\n{:?}\nWith mask\n{:?}",
                    witness.dimension, witness.repr, witness.mask
                );

                are_equal(&witness.repr, &new_p.repr);
                are_equal(&witness.mask, &new_p.mask);
                are_equal(&mask_witness, &mask);
                are_equal(&anti_mask_witness, &anti_mask);

                // println!("new_p repr:\n{:?}", new_p.repr);
                // println!("new_p mask:\n{:?}", new_p.mask);
                // println!("anti-mask:\n{:?}", anti_mask);

                // println!("	- declined polyomino (added at {i},{j}):");
                // println!("{new_p:?}");
                polyominoes.push(new_p);
            }
        }
    }

    // FIXME: doesn't work when reordering polyominoes,
    // due to the mask being more generic for the first ones generated
    // IDEA: reset the mask if the polyomino was not seen before (but the mask would not be useful then)
    //      Or take the mask with the higher count among rotations,
    //      but this means that all polyomnioes of some square count must be processed before starting the next square count
    // polyominoes.reverse();
    polyominoes
}

pub fn smallest_rotation<T>(p: Polyomino<T>) -> (Polyomino<T>, Rotation)
where
    T: Grid,
{
    // Take smallest dimension first
    if p.dimension.0 < p.dimension.1 {
        // Only compare rotation 0 and rotation 180
        let rotated_board = p.repr.rotate(p.dimension, Rotation::R180);
        if p.repr < rotated_board {
            (p, Rotation::R0)
        } else {
            (
                Polyomino {
                    square_count: p.square_count,
                    dimension: p.dimension,
                    repr: rotated_board,
                    mask: p.mask.rotate(p.dimension, Rotation::R180),
                },
                Rotation::R180,
            )
        }
    } else if p.dimension.0 > p.dimension.1 {
        // Compare rotations 90 and 270
        let board_90 = p.repr.rotate(p.dimension, Rotation::R90);
        let board_270 = p.repr.rotate(p.dimension, Rotation::R270);
        if board_90 < board_270 {
            (
                Polyomino {
                    square_count: p.square_count,
                    dimension: (p.dimension.1, p.dimension.0),
                    repr: board_90,
                    mask: p.mask.rotate(p.dimension, Rotation::R90),
                },
                Rotation::R90,
            )
        } else {
            (
                Polyomino {
                    square_count: p.square_count,
                    dimension: (p.dimension.1, p.dimension.0),
                    repr: board_270,
                    mask: p.mask.rotate(p.dimension, Rotation::R270),
                },
                Rotation::R270,
            )
        }
    } else {
        // Dimensions are equal
        // Compare all rotations
        let board_90 = p.repr.rotate(p.dimension, Rotation::R90);
        let board_180 = p.repr.rotate(p.dimension, Rotation::R180);
        let board_270 = p.repr.rotate(p.dimension, Rotation::R270);

        // Comparison by pairs : (p, board_180) and (board_90, board_270)
        let (smallest_1, rot1) = {
            if p.repr < board_180 {
                (p.repr, Rotation::R0)
            } else {
                (board_180, Rotation::R180)
            }
        };
        let (smallest_2, rot2) = {
            if board_90 < board_270 {
                (board_90, Rotation::R90)
            } else {
                (board_270, Rotation::R270)
            }
        };

        // Then compare the smallest of each pair
        if smallest_1 < smallest_2 {
            (
                Polyomino {
                    square_count: p.square_count,
                    dimension: p.dimension,
                    repr: smallest_1,
                    mask: p.mask.rotate(p.dimension, rot1),
                },
                rot1,
            )
        } else {
            (
                Polyomino {
                    square_count: p.square_count,
                    dimension: p.dimension,
                    repr: smallest_2,
                    mask: p.mask.rotate(p.dimension, rot2),
                },
                rot2,
            )
        }
    }
}
