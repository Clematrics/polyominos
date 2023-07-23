use std::fmt::{self, Debug};

use crate::board::*;
use crate::rotation::Rotation;

#[derive(Copy, Clone)]
pub struct Polyomino {
    pub square_count: u8,
    pub dimension: (u8, u8),
    pub repr: Board,
    pub mask: Board,
}

impl Debug for Polyomino {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for j in (0..SIZE).rev() {
            for i in 0..SIZE {
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

                f.write_str(&to_print).unwrap();
            }
            f.write_str("\n").unwrap();
        }

        Ok(())
    }
}

impl Polyomino {
    pub fn trivial() -> Self {
        let repr = {
            let mut board = Board::new();
            board.set(1, 1);
            board
        };
        let mask = {
            let mut board = Board::new();
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

    pub fn from(dimension: (u8, u8), repr: Board, mask: Board) -> Self {
        Self {
            square_count: repr.count() as u8,
            dimension,
            repr,
            mask,
        }
    }

    pub fn add_square(&mut self, mut x: usize, mut y: usize, anti_mask: &Board) {
        self.square_count += 1;
        // The anti_mask is not shifted like the repr and mask boards, for efficiency reasons
        // Thus, we need offsets on coordinates to access the anti_mask
        println!("Before {x}, {y}");
        let (mut x_offset, mut y_offset) = (0, 0);

        if x + 1 == self.dimension.0 as usize {
            self.dimension.0 += 1;
        }
        if y + 1 == self.dimension.1 as usize {
            self.dimension.1 += 1;
        }
        if x == 0 {
            x_offset = 1;
            self.repr.shift_x(1);
            self.mask.shift_x(1);
            self.dimension.0 += 1;
            x += 1;
        }
        if y == 0 {
            y_offset = 1;
            self.repr.shift_y(1);
            self.mask.shift_y(1);
            self.dimension.1 += 1;
            y += 1;
        }
        println!("After {x}, {y}");
        // NOTE: there is no need to optimise for a shift both in x and y,
        // since no square can be added here (otherwise it would not be connected)

        // IDEA: localize addition of a square rather than subtracting a whole mask
        self.repr.set(x, y);
        self.mask.unset(x, y);
        for (x, y) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)].into_iter() {
            let anti_mask = if x == 0 || y == 0 {
                false
            } else {
                anti_mask.get(x - x_offset, y - y_offset)
            };
            if !self.repr.get(x, y) && !anti_mask {
                println!("Set mask at {x}, {y}");
                self.mask.set(x, y)
            };
        }
    }
}

/// Return all polyominoes that can be created by adding a square to this polyomino, excluding positions out of the mask
pub fn decline(p: &Polyomino) -> Vec<Polyomino> {
    let mut polyominoes = vec![];
    // println!("From polyomino:");
    // println!("{p:?}");

    let mut anti_mask = Board::new();
    for i in 0..SIZE {
        for j in 0..SIZE {
            if p.mask.get(i, j) {
                // add a new polyomino
                let mut new_p = p.clone();
                new_p.add_square(i, j, &anti_mask);
                anti_mask.set(i, j);

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

pub fn smallest_rotation(p: Polyomino) -> Polyomino {
    // Take smallest dimension first
    if p.dimension.0 < p.dimension.1 {
        // Only compare rotation 0 and rotation 180
        let rotated_board = p.repr.rotate(p.dimension, &Rotation::R180);
        println!(
            "180 rotation:\n{:?}",
            Polyomino::from(p.dimension, rotated_board, Board::new())
        );
        if p.repr < rotated_board {
            p
        } else {
            Polyomino {
                square_count: p.square_count,
                dimension: p.dimension,
                repr: rotated_board,
                mask: p.mask.rotate(p.dimension, &Rotation::R180),
            }
        }
    } else if p.dimension.0 > p.dimension.1 {
        // Compare rotations 90 and 270
        let board_90 = p.repr.rotate(p.dimension, &Rotation::R90);
        let board_270 = p.repr.rotate(p.dimension, &Rotation::R270);
        println!(
            "90 rotation:\n{:?}",
            Polyomino::from(p.dimension, board_90, Board::new())
        );
        println!(
            "270 rotation:\n{:?}",
            Polyomino::from(p.dimension, board_270, Board::new())
        );
        if board_90 < board_270 {
            Polyomino {
                square_count: p.square_count,
                dimension: (p.dimension.1, p.dimension.0),
                repr: board_90,
                mask: p.mask.rotate(p.dimension, &Rotation::R90),
            }
        } else {
            Polyomino {
                square_count: p.square_count,
                dimension: (p.dimension.1, p.dimension.0),
                repr: board_270,
                mask: p.mask.rotate(p.dimension, &Rotation::R270),
            }
        }
    } else {
        // Dimensions are equal
        // Compare all rotations
        let board_90 = p.repr.rotate(p.dimension, &Rotation::R90);
        let board_180 = p.repr.rotate(p.dimension, &Rotation::R180);
        let board_270 = p.repr.rotate(p.dimension, &Rotation::R270);

        println!(
            "90 rotation:\n{:?}",
            Polyomino::from(p.dimension, board_90, Board::new())
        );
        println!(
            "180 rotation:\n{:?}",
            Polyomino::from(p.dimension, board_180, Board::new())
        );
        println!(
            "270 rotation:\n{:?}",
            Polyomino::from(p.dimension, board_270, Board::new())
        );

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
            Polyomino {
                square_count: p.square_count,
                dimension: p.dimension,
                repr: smallest_1,
                mask: p.mask.rotate(p.dimension, &rot1),
            }
        } else {
            Polyomino {
                square_count: p.square_count,
                dimension: p.dimension,
                repr: smallest_2,
                mask: p.mask.rotate(p.dimension, &rot2),
            }
        }
    }
}
