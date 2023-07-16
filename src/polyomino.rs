use std::fmt::{self, Debug};

use crate::board::{self, *};
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
        for j in 0..8 {
            for i in 0..8 {
                let str = {
                    if self.repr[i][j] {
                        if self.mask[i][j] {
                            "!"
                        } else {
                            "O"
                        }
                    } else {
                        if self.mask[i][j] {
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
    pub fn add_square(&mut self, x: usize, y: usize, anti_mask: &Board) {
        self.square_count += 1;
        // IDEA: localize addition of a square rather than subtracting a whole mask
        self.repr[x][y] = true;

        let (x, y, anti_mask) = self.shift(x, y, anti_mask); // x and y are now guaranteed to be in [(1,1), dimension]
        self.mask[x + 1][y] = true;
        self.mask[x - 1][y] = true;
        self.mask[x][y + 1] = true;
        self.mask[x][y - 1] = true;
        self.mask.sub(&self.repr);
        self.mask.sub(&anti_mask);
    }

    fn shift(&mut self, mut x: usize, mut y: usize, anti_mask: &Board) -> (usize, usize, Board) {
        // Adjust the dimension & boards if the extended square is on the board (so that the mask is still inside the boundaries)
        // IDEA: make mask boudaries larger, such that each boundary of the representation always has a square touching it

        let mut anti_mask = anti_mask.clone();

        // This take into account the case where self.dimension.{0, 1} == 1
        if x + 1 == self.dimension.0 as usize {
            // augment the X boundary
            self.dimension.0 += 1;
        }
        if x == 0 {
            self.repr.copy_within(0..board::SIZE - 1, 1);
            self.mask.copy_within(0..board::SIZE - 1, 1);
            anti_mask.copy_within(0..board::SIZE - 1, 1);
            self.repr[0] = [false; board::SIZE];
            self.mask[0] = [false; board::SIZE];
            anti_mask[0] = [false; board::SIZE];

            self.dimension.0 += 1;
            x += 1;
        }
        if y + 1 == self.dimension.1 as usize {
            self.dimension.1 += 1;
        }
        if y == 0 {
            for i in 0..SIZE {
                self.repr[i].copy_within(0..SIZE - 1, 1);
                self.mask[i].copy_within(0..SIZE - 1, 1);
                anti_mask[i].copy_within(0..SIZE - 1, 1);
                self.repr[i][0] = false;
                self.mask[i][0] = false;
                anti_mask[i][0] = false;
            }

            self.dimension.1 += 1;
            y += 1;
        }

        (x, y, anti_mask)
    }
}

/// Return all polyominoes that can be created by adding a square to this polyomino, excluding positions out of the mask
pub fn decline(p: &Polyomino) -> Vec<Polyomino> {
    let mut polyominoes = vec![];
    // println!("From polyomino:");
    // println!("{p:?}");

    let mut anti_mask = Board::new(false);
    for i in 0..SIZE {
        for j in 0..SIZE {
            if p.mask[i][j] {
                // add a new polyomino
                let mut new_p = p.clone();
                new_p.add_square(i, j, &anti_mask);
                anti_mask[i][j] = true;

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
        let rotated_board = rotate_board(&p.repr, p.dimension, &Rotation::R180);
        if p.repr < rotated_board {
            p
        } else {
            Polyomino {
                square_count: p.square_count,
                dimension: p.dimension,
                repr: rotated_board,
                mask: rotate_board(&p.mask, p.dimension, &Rotation::R180),
            }
        }
    } else if p.dimension.0 > p.dimension.1 {
        // Compare rotations 90 and 270
        let board_90 = rotate_board(&p.repr, p.dimension, &Rotation::R90);
        let board_270 = rotate_board(&p.repr, p.dimension, &Rotation::R270);
        if board_90 < board_270 {
            Polyomino {
                square_count: p.square_count,
                dimension: (p.dimension.1, p.dimension.0),
                repr: board_90,
                mask: rotate_board(&p.mask, p.dimension, &Rotation::R90),
            }
        } else {
            Polyomino {
                square_count: p.square_count,
                dimension: (p.dimension.1, p.dimension.0),
                repr: board_270,
                mask: rotate_board(&p.mask, p.dimension, &Rotation::R270),
            }
        }
    } else {
        // Dimensions are equal
        // Compare all rotations
        let board_90 = rotate_board(&p.repr, p.dimension, &Rotation::R90);
        let board_180 = rotate_board(&p.repr, p.dimension, &Rotation::R180);
        let board_270 = rotate_board(&p.repr, p.dimension, &Rotation::R270);

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
                mask: rotate_board(&p.mask, p.dimension, &rot1),
            }
        } else {
            Polyomino {
                square_count: p.square_count,
                dimension: p.dimension,
                repr: smallest_2,
                mask: rotate_board(&p.mask, p.dimension, &rot2),
            }
        }
    }
}
