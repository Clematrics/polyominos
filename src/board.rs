use std::{fmt::Debug, ops::BitOrAssign};

// #[repr(transparent)]
// struct Block(u16);

// impl Deref for Block {
//     type Target = u16;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl DerefMut for Block {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// impl Block {
//     fn rotate(&self, rot: Rotation) -> Self {
//         match rot {
//             Rotation::R0 => Block(self.0),
//             Rotation::R90 => {
//                 let x = self.0;
//                 x.
//             },
//             Rotation::R180 => Block(self.0.reverse_bits()),
//             Rotation::R270 => self,
//         }
//     }
// }

use crate::rotation::Rotation;

pub const SIZE: usize = 32;

type InnerBoard = [u32; SIZE];

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Board {
    /// A board representation. Each element represents a column
    /// Thus, to get the square at (x, y), do (board[x] >> y) & 1
    board: InnerBoard,
}

impl Board {
    pub fn new() -> Self {
        Self { board: [0; SIZE] }
    }

    pub fn set(&mut self, x: usize, y: usize) {
        self.board[x] |= 1 << y;
    }

    pub fn unset(&mut self, x: usize, y: usize) {
        self.board[x] &= u32::MAX - (1 << y);
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        if y >= SIZE {
            panic!("Whooat??! {x}, {y}")
        }
        ((self.board[x] >> y) & 1) != 0
    }

    pub fn sub(&mut self, other: &Self) {
        for x in 0..SIZE {
            self.board[x] &= !other.board[x];
        }
    }

    pub fn count(&self) -> u32 {
        self.board.iter().map(|column| column.count_ones()).sum()
    }

    pub fn rotate(&self, dim: (u8, u8), r: &Rotation) -> Board {
        match r {
            Rotation::R0 => self.clone(),
            Rotation::R90 => {
                let mut new = Self::new();
                rotate_90(&self.board, &mut new.board);
                // new.shift_y((SIZE as isize - dim.1 as isize) as i8);
                new
            }
            Rotation::R180 => {
                // Simply reverse the array and reverse the bit order of each column
                let mut new = Self::new();
                for x in 0..SIZE {
                    new.board[x] = self.board[SIZE - 1 - x].reverse_bits();
                }
                new.shift_x((SIZE as isize - dim.0 as isize) as i8);
                new.shift_y((SIZE as isize - dim.1 as isize) as i8);
                new
            }
            Rotation::R270 => {
                // Same thing as a 90 rotation
                let mut new = Self::new();
                rotate_270(&self.board, &mut new.board);
                new.shift_x((SIZE as isize - dim.0 as isize) as i8);
                new
            }
        }
    }

    pub fn shift_x(&mut self, amount: i8) {
        if amount.abs() as isize > SIZE as isize {
            panic!("The shift amount is too big!");
        }

        match amount.cmp(&0) {
            std::cmp::Ordering::Equal => (),
            std::cmp::Ordering::Less => {
                let amount = (-amount) as usize;

                self.board.copy_within(amount..SIZE, 0);
                self.board[(SIZE - amount)..SIZE].fill(0);
            }
            std::cmp::Ordering::Greater => {
                let amount = amount as usize;

                self.board.copy_within(0..(SIZE - amount), amount);
                self.board[0..amount].fill(0);
            }
        }
    }

    pub fn shift_y(&mut self, amount: i8) {
        if amount.abs() as isize > SIZE as isize {
            panic!("The shift amount is too big!");
        }

        for x in 0..SIZE {
            self.board[x] <<= amount;
        }
    }
}

impl BitOrAssign for Board {
    fn bitor_assign(&mut self, rhs: Self) {
        for x in 0..SIZE {
            self.board[x] |= rhs.board[x]
        }
    }
}

#[repr(transparent)]
struct M256([i32; 8]);
use std::arch::x86_64::__m256i;
fn read(m: __m256i) -> M256 {
    use std::arch::x86_64::*;
    let mut res = [0i32; 8];
    unsafe {
        let store_mask = _mm256_set1_epi32((0x80000000 as u32) as i32); // To avoid warning
        _mm256_maskstore_epi32(res.as_mut_ptr(), store_mask, m);
    }
    M256(res)
}

impl Debug for M256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:0>8X} {:0>8X} {:0>8X} {:0>8X} {:0>8X} {:0>8X} {:0>8X} {:0>8X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], self.0[6], self.0[7]
        ))
    }
}

fn rotate_90(from: &InnerBoard, to: &mut InnerBoard) {
    // This gets funny
    // For a 90 trigonometric rotation
    // - all MSB of the unrotated columns must be in column 0
    //   board[0].msb moves to board[0].lsb
    //   board[SIZE].msb moves to board[0].msb

    unsafe {
        use std::arch::x86_64::*;

        let first_shuffle = _mm256_set_epi32(
            0x0F_0B_07_03,
            0x0E_0A_06_02,
            0x0D_09_05_01,
            0x0C_08_04_00,
            // Lane split
            0x0F_0B_07_03,
            0x0E_0A_06_02,
            0x0D_09_05_01,
            0x0C_08_04_00,
        );
        let second_shuffle = _mm256_set_epi32(0x7, 0x3, 0x6, 0x2, 0x5, 0x1, 0x4, 0x0);

        // strips[0] is the highest byte. Contains most significant bytes of columns, so unrotated board[:][31:24]
        // strips[1] the next highest byte. Contains unrotated board[:][23:16]
        // strips[2] the next highest byte. Contains unrotated board[:][15:8]
        // strips[3] is the lowest byte. Contains unrotated board[:][7:0]
        let mut strips = [[0i32; 8]; 4];

        for x in 0..4 {
            let col = _mm256_set_epi32(
                from[x * 8 + 0] as i32,
                from[x * 8 + 1] as i32,
                from[x * 8 + 2] as i32,
                from[x * 8 + 3] as i32,
                from[x * 8 + 4] as i32,
                from[x * 8 + 5] as i32,
                from[x * 8 + 6] as i32,
                from[x * 8 + 7] as i32,
            );

            // println!("col: {:?}", read(col));

            // The highest byte of columns of each 128 bit parts are joined together
            // From
            // |    C0   |    C1   |    C2   |    C3   ||    C4   |    C5   |    C6   |    C7   |
            // | H h l L | H h l L | H h l L | H h l L || H h l L | H h l L | H h l L | H h l L |
            // To
            // | C C C C | C C C C | C C C C | C C C C || C C C C | C C C C | C C C C | C C C C |
            // | 0 1 2 3 | 0 1 2 3 | 0 1 2 3 | 0 1 2 3 || 4 5 6 7 | 4 5 6 7 | 4 5 6 7 | 4 5 6 7 |
            // | H H H H | h h h h | l l l l | L L L L || H H H H | h h h h | l l l l | L L L L |
            let half_ordered = _mm256_shuffle_epi8(col, first_shuffle);

            // Permuting each block such that all bytes of the same importance are contiguous
            // From
            // | C C C C | C C C C | C C C C | C C C C || C C C C | C C C C | C C C C | C C C C |
            // | 0 1 2 3 | 0 1 2 3 | 0 1 2 3 | 0 1 2 3 || 4 5 6 7 | 4 5 6 7 | 4 5 6 7 | 4 5 6 7 |
            // | H H H H | h h h h | l l l l | L L L L || H H H H | h h h h | l l l l | L L L L |
            // From
            // | C C C C | C C C C | C C C C | C C C C | C C C C | C C C C | C C C C | C C C C |
            // | 0 1 2 3 | 4 5 6 7 | 0 1 2 3 | 4 5 6 7 | 0 1 2 3 | 4 5 6 7 | 0 1 2 3 | 4 5 6 7 |
            // | H H H H | H H H H | h h h h | h h h h | l l l l | l l l l | L L L L | L L L L |
            let ordered = _mm256_permutevar8x32_epi32(half_ordered, second_shuffle);

            // println!("ordered: {:?}", read(ordered));

            let mut res = [0i32; 8];
            let store_mask = _mm256_set1_epi32((0x80000000 as u32) as i32); // To avoid warning
            _mm256_maskstore_epi32(res.as_mut_ptr(), store_mask, ordered);
            // TODO: stored as little indian

            for i in 0..8 {
                strips[4 - 1 - i / 2][2 * x + 1 - i % 2] = res[i];
            }
        }

        // println!("Strips:");
        // for l in strips.iter() {
        //     for e in l.iter() {
        //         print!("{:0>8X}", e);
        //     }
        //     println!("");
        // }

        // Each increment in 'a' counts for 8 columns
        for (a, strip) in strips.iter().enumerate() {
            let strip = _mm256_set_epi32(
                strip[0], strip[1], strip[2], strip[3], strip[4], strip[5], strip[6], strip[7],
            );

            for (b, &pattern) in [0x80 as u8 as i8, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01]
                .iter()
                .enumerate()
            {
                let mask = _mm256_set1_epi8(pattern); // Broadcast the pattern to each byte
                let ands = _mm256_and_si256(mask, strip); // Select the bit of the pattern in each byte
                let bits = _mm256_cmpeq_epi8(ands, mask); // If a byte is non-zero, sets it to 0xFF, 0x00 otherwise
                                                          // println!(
                                                          //     "Strip {}: cmp res with mask {:0>2x}: {:?}",
                                                          //     a,
                                                          //     pattern,
                                                          //     read(bits)
                                                          // );
                let coln = _mm256_movemask_epi8(bits) as u32; // Each bit of the result is taken from the MSB of the corresponding byte
                                                              // println!("Strip {}: Res coln with mask {:0>2x}: {}", a, pattern, coln);
                to[a * 8 + b] = coln.reverse_bits(); // The MSB corresponds to the old column 0, so we need to reverse it
            }
        }
    };
}

fn rotate_270(from: &InnerBoard, to: &mut InnerBoard) {
    // Same logic as rotation_90.
    // The only change is that we apply a 180 rotation after.
    // This is equivalent of loading the new columns in reverse order and without flipping their bit order

    unsafe {
        use std::arch::x86_64::*;

        let first_shuffle = _mm256_set_epi32(
            0x0F_0B_07_03,
            0x0E_0A_06_02,
            0x0D_09_05_01,
            0x0C_08_04_00,
            // Lane split
            0x0F_0B_07_03,
            0x0E_0A_06_02,
            0x0D_09_05_01,
            0x0C_08_04_00,
        );
        let second_shuffle = _mm256_set_epi32(0x7, 0x3, 0x6, 0x2, 0x5, 0x1, 0x4, 0x0);

        // strips[0] is the highest byte. Contains most significant bytes of columns, so unrotated board[:][31:24]
        // strips[1] the next highest byte. Contains unrotated board[:][23:16]
        // strips[2] the next highest byte. Contains unrotated board[:][15:8]
        // strips[3] is the lowest byte. Contains unrotated board[:][7:0]
        let mut strips = [[0i32; 8]; 4];

        for x in 0..4 {
            let col = _mm256_set_epi32(
                from[x * 8 + 0] as i32,
                from[x * 8 + 1] as i32,
                from[x * 8 + 2] as i32,
                from[x * 8 + 3] as i32,
                from[x * 8 + 4] as i32,
                from[x * 8 + 5] as i32,
                from[x * 8 + 6] as i32,
                from[x * 8 + 7] as i32,
            );

            // println!("col: {:?}", read(col));

            // The highest byte of columns of each 128 bit parts are joined together
            // From
            // |    C0   |    C1   |    C2   |    C3   ||    C4   |    C5   |    C6   |    C7   |
            // | H h l L | H h l L | H h l L | H h l L || H h l L | H h l L | H h l L | H h l L |
            // To
            // | C C C C | C C C C | C C C C | C C C C || C C C C | C C C C | C C C C | C C C C |
            // | 0 1 2 3 | 0 1 2 3 | 0 1 2 3 | 0 1 2 3 || 4 5 6 7 | 4 5 6 7 | 4 5 6 7 | 4 5 6 7 |
            // | H H H H | h h h h | l l l l | L L L L || H H H H | h h h h | l l l l | L L L L |
            let half_ordered = _mm256_shuffle_epi8(col, first_shuffle);

            // Permuting each block such that all bytes of the same importance are contiguous
            // From
            // | C C C C | C C C C | C C C C | C C C C || C C C C | C C C C | C C C C | C C C C |
            // | 0 1 2 3 | 0 1 2 3 | 0 1 2 3 | 0 1 2 3 || 4 5 6 7 | 4 5 6 7 | 4 5 6 7 | 4 5 6 7 |
            // | H H H H | h h h h | l l l l | L L L L || H H H H | h h h h | l l l l | L L L L |
            // From
            // | C C C C | C C C C | C C C C | C C C C | C C C C | C C C C | C C C C | C C C C |
            // | 0 1 2 3 | 4 5 6 7 | 0 1 2 3 | 4 5 6 7 | 0 1 2 3 | 4 5 6 7 | 0 1 2 3 | 4 5 6 7 |
            // | H H H H | H H H H | h h h h | h h h h | l l l l | l l l l | L L L L | L L L L |
            let ordered = _mm256_permutevar8x32_epi32(half_ordered, second_shuffle);

            // println!("ordered: {:?}", read(ordered));

            let mut res = [0i32; 8];
            let store_mask = _mm256_set1_epi32((0x80000000 as u32) as i32); // To avoid warning
            _mm256_maskstore_epi32(res.as_mut_ptr(), store_mask, ordered);
            // TODO: stored as little indian

            for i in 0..8 {
                strips[4 - 1 - i / 2][2 * x + 1 - i % 2] = res[i];
            }
        }

        // println!("Strips:");
        // for l in strips.iter() {
        //     for e in l.iter() {
        //         print!("{:0>8X}", e);
        //     }
        //     println!("");
        // }

        // Each increment in 'a' counts for 8 columns
        for (a, strip) in strips.iter().enumerate() {
            let strip = _mm256_set_epi32(
                strip[0], strip[1], strip[2], strip[3], strip[4], strip[5], strip[6], strip[7],
            );

            for (b, &pattern) in [0x80 as u8 as i8, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01]
                .iter()
                .enumerate()
            {
                let mask = _mm256_set1_epi8(pattern); // Broadcast the pattern to each byte
                let ands = _mm256_and_si256(mask, strip); // Select the bit of the pattern in each byte
                let bits = _mm256_cmpeq_epi8(ands, mask); // If a byte is non-zero, sets it to 0xFF, 0x00 otherwise
                                                          // println!(
                                                          //     "Strip {}: cmp res with mask {:0>2x}: {:?}",
                                                          //     a,
                                                          //     pattern,
                                                          //     read(bits)
                                                          // );
                let coln = _mm256_movemask_epi8(bits) as u32; // Each bit of the result is taken from the MSB of the corresponding byte
                                                              // println!("Strip {}: Res coln with mask {:0>2x}: {}", a, pattern, coln);
                to[SIZE - 1 - (a * 8 + b)] = coln; // The only line that changes with rotate_90
            }
        }
    };
}
