use std::fmt::{Debug, Write};
use std::hash::Hash;
use std::iter;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

use crate::grid::Grid;
use crate::rotation::Rotation;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Block(u16);

impl Block {
    fn get(&self, x: usize, y: usize) -> bool {
        // To get the (x%4, y%4) bit, we proceed by selecting
        // the good line and good column using masks and then and them
        let y_mask = 0x1111 << y;
        let x_mask = 0x000F << (x * 4);
        let bit = self.0 & x_mask & y_mask;
        bit != 0

        // TODO: compare to self.0 & (1 << (x * 4 | y)) != 0
    }

    fn set(&self, x: usize, y: usize) -> Self {
        // We find the good bit by anding a line mask and
        // a column mask
        let y_mask = 0x1111 << y;
        let x_mask = 0x000F << (x * 4);
        let bit = x_mask & y_mask;

        Self(self.0 | bit)

        // TODO: compare to self.0 | (1 << (x * 4 | y))
    }

    fn unset(&self, x: usize, y: usize) -> Self {
        // We find the good bit by anding a line mask and
        // a column mask. Then we inverse the mask to keep only
        // the other bits
        let y_mask = 0x1111 << (y);
        let x_mask = 0x000F << ((x) * 4);
        let mask = !(x_mask & y_mask);

        Self(self.0 & mask)

        // TODO: compare to self.0 & (0xF...FE << (x * 4 | y))
    }

    fn is_empty(&self) -> bool {
        self.0 == 0
    }

    fn count(&self) -> u32 {
        self.0.count_ones()
    }

    fn get_bounding_box(&self) -> (usize, usize) {
        if self.is_empty() {
            return (0, 0);
        }

        let mut max_x = 0;
        let mut max_y = 0;

        for x in 0..4 {
            for y in 0..4 {
                if self.get(x, y) {
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }
        }

        // We return +1 to get the bounding box rather than the max index
        (max_x + 1, max_y + 1)
    }

    /// shift the block on the X axis, returning the part going outside
    fn shift_x(&self, n: isize, shift_in: u16) -> (Self, u16) {
        if n == 0 {
            (Self(self.0), 0)
        } else if n > 0 {
            // Using u32 for less operations on a single block
            let tmp = ((self.0 as u32) << (4 * n)) | shift_in as u32;
            let block = Block((tmp & 0xFFFF) as u16);
            let shift_out = (tmp >> 16) as u16;
            (block, shift_out)

            // Using i16: (can be vectorized using twice less register space)
            // let block = self.0;
            // let new_block = (block << (4 * n)) | shift_in;
            // let shift_out = block >> (16 - 4 * n);
            // (new_block, shift_out)
        } else
        /* n < 0 */
        {
            // Using u32 for less operations on a single block
            let tmp = (((self.0 as u32) << 16) >> (4 * -n)) | ((shift_in as u32) << 16);
            let block = Block((tmp >> 16) as u16);
            let shift_out = (tmp & 0xFFFF) as u16;
            (block, shift_out)

            // Using i16: (can be vectorized using twice less register space)
            // let block = self.0;
            // let new_block = (block >> (4 * n)) | shift_in;
            // let shift_out = block << (16 - 4 * n);
            // (new_block, shift_out)
        }
    }

    /// shift the block on the Y axis, returning the part going outside
    fn shift_y(&self, n: isize, shift_in: u16) -> (Self, u16) {
        if n == 0 {
            (Self(self.0), 0)
        } else if n > 0 {
            let top_row_mask = unsafe {
                iter::repeat(0b0001_0001_0001_0001u16)
                    .enumerate()
                    .map(|(i, mask)| mask << i)
                    .take(n as usize)
                    .reduce(|acc, oth| acc | oth)
                    .unwrap_unchecked()
            };
            let block_mask = unsafe {
                iter::repeat(0b1110_1110_1110_1110u16)
                    .enumerate()
                    .map(|(i, mask)| mask << i)
                    .take(n as usize)
                    .reduce(|acc, oth| acc & oth)
                    .unwrap_unchecked()
            };
            let block = self.0;
            let new_block = (block << n) & block_mask | shift_in;
            let shift_out = (block >> (4 - n)) & top_row_mask;
            (Self(new_block), shift_out)
        } else
        /* n < 0 */
        {
            let top_row_mask = unsafe {
                iter::repeat(0b1000_1000_1000_1000u16)
                    .enumerate()
                    .map(|(i, mask)| mask >> i)
                    .take(-n as usize)
                    .reduce(|acc, oth| acc | oth)
                    .unwrap_unchecked()
            };
            let block_mask = unsafe {
                iter::repeat(0b0111_0111_0111_0111u16)
                    .enumerate()
                    .map(|(i, mask)| mask >> i)
                    .take(-n as usize)
                    .reduce(|acc, oth| acc & oth)
                    .unwrap_unchecked()
            };
            let block = self.0;
            let new_block = (block >> (-n)) & block_mask | shift_in;
            let shift_out = (block << (n + 4)) & top_row_mask;
            (Self(new_block), shift_out)
        }
    }

    fn batch_2_rotate_90_avx2(chunk: [u16; 2]) -> (u16, u16) {
        use std::arch::x86_64::*;

        unsafe {
            let mm_high = _mm_set1_epi16(x1 as i16);
            let mm_low = _mm_set1_epi16(x2 as i16);

            let mm = _mm256_set_m128i(mm_high, mm_low);

            #[allow(overflowing_literals)]
            let mask_part = _mm_set_epi64x(0x8080404020201010, 0x0808040402020101);
            let mask = _mm256_set_m128i(mask_part, mask_part);

            let bits = _mm256_and_si256(mm, mask);
            let cmp = _mm256_cmpeq_epi8(bits, mask);

            let shuffler_part = _mm_set_epi8(9, 1, 8, 0, 11, 3, 10, 2, 13, 5, 12, 4, 15, 7, 14, 6);
            let shuffler = _mm256_set_m128i(shuffler_part, shuffler_part);

            let shuffled = _mm256_shuffle_epi8(cmp, shuffler);

            let result = _mm256_movemask_epi8(shuffled) as u32;
            let result_high = (result >> 16) as u16;
            let result_low = (result & 0xFFFF) as u16;
            (result_high, result_low)
        }
    }

    fn batch_2_rotate_180_avx2(x1: u16, x2: u16) -> (u16, u16) {
        use std::arch::x86_64::*;

        unsafe {
            let mm_high = _mm_set1_epi16(x1 as i16);
            let mm_low = _mm_set1_epi16(x2 as i16);

            let mm = _mm256_set_m128i(mm_high, mm_low);

            #[allow(overflowing_literals)]
            let mask_part = _mm_set_epi64x(0x8080404020201010, 0x0808040402020101);
            let mask = _mm256_set_m128i(mask_part, mask_part);

            let bits = _mm256_and_si256(mm, mask);
            let cmp = _mm256_cmpeq_epi8(bits, mask);

            let shuffler_part = _mm_set_epi8(0, 2, 4, 6, 8, 10, 12, 14, 1, 3, 5, 7, 9, 11, 13, 15);
            let shuffler = _mm256_set_m128i(shuffler_part, shuffler_part);

            let shuffled = _mm256_shuffle_epi8(cmp, shuffler);

            let result = _mm256_movemask_epi8(shuffled) as u32;
            let result_high = (result >> 16) as u16;
            let result_low = (result & 0xFFFF) as u16;
            (result_high, result_low)
        }
    }

    fn batch_2_rotate_270_avx2(x1: u16, x2: u16) -> (u16, u16) {
        use std::arch::x86_64::*;

        unsafe {
            let mm_high = _mm_set1_epi16(x1 as i16);
            let mm_low = _mm_set1_epi16(x2 as i16);

            let mm = _mm256_set_m128i(mm_high, mm_low);

            #[allow(overflowing_literals)]
            let mask_part = _mm_set_epi64x(0x8080404020201010, 0x0808040402020101);
            let mask = _mm256_set_m128i(mask_part, mask_part);

            let bits = _mm256_and_si256(mm, mask);
            let cmp = _mm256_cmpeq_epi8(bits, mask);

            let shuffler_part = _mm_set_epi8(6, 14, 7, 15, 4, 12, 5, 13, 2, 10, 3, 11, 0, 8, 1, 9);
            let shuffler = _mm256_set_m128i(shuffler_part, shuffler_part);

            let shuffled = _mm256_shuffle_epi8(cmp, shuffler);

            let result = _mm256_movemask_epi8(shuffled) as u32;
            let result_high = (result >> 16) as u16;
            let result_low = (result & 0xFFFF) as u16;
            (result_high, result_low)
        }
    }

    // fn batch_2_rotate_90_avx512(x1: u16, x2: u16, x3: u16, x4: u16) -> (u16, u16) {
    //     use std::arch::x86_64::*;

    //     unsafe {
    //         let mm_high = _mm_set1_epi16(x1 as i16);
    //         let mm_low = _mm_set1_epi16(x2 as i16);

    //         // let x1 = (x1 as u16) as u64;
    //         // let x2 = (x2 as u16) as u64;
    //         // let x3 = (x3 as u16) as u64;
    //         // let x4 = (x4 as u16) as u64;
    //         // let ex = ((x1 << 48) | (x2 << 32) | (x3 << 16) | x4) as i64;

    //         // unsafe {
    //         //     let mm = _mm512_set1_epi64(ex);
    //         //     _mm512_shuffle_epi8
    //         //     let res = _mm512_cmpeq_epi8_mask(mm, mm);
    //         //     (res & 0xFFFF) as i16
    //         // }

    //         let mm = _mm256_set_m128i(mm_high, mm_low);

    //         #[allow(overflowing_literals)]
    //         let mask_part = _mm_set_epi64x(0x8080404020201010, 0x0808040402020101);
    //         let mask = _mm256_set_m128i(mask_part, mask_part);

    //         let bits = _mm256_and_si256(mm, mask);
    //         let cmp = _mm256_cmpeq_epi8(bits, mask);

    //         let shuffler_part = _mm_set_epi8(9, 1, 8, 0, 11, 3, 10, 2, 13, 5, 12, 4, 15, 7, 14, 6);
    //         let shuffler = _mm256_set_m128i(shuffler_part, shuffler_part);

    //         let shuffled = _mm512_shuffle_epi32(cmp, shuffler);

    //         let result = _mm256_movemask_epi8(shuffled) as u32;
    //         let result_high = (result >> 16) as u16;
    //         let result_low = (result & 0xFFFF) as u16;
    //         (result_high, result_low)
    //     }
    // }

    fn rotate_270(x: u16) -> u16 {
        // Rotating by 270 degree
        // We need to map bits of index ... to bits of index
        // - [0, 1, 2, 3] -> [3, 7, 11, 15]
        // - [4, 5, 6, 7] -> [2, 6, 10, 14]
        // - [8, 9, 10, 11] -> [1, 5, 9, 13]
        // - [12, 13, 14, 15] -> [0, 4, 8, 12]
        let mut new_block = 0u16;

        // All variables named bits_xx_xx have these bits from the
        // original block in contiguous form in the lowest bits of
        // the variable.
        {
            let bits_2_0 = x & 0x7;
            let bit_3_at_bit_15 = (x & 0x0008) << 12;

            new_block |= ((bits_2_0 * 0b1001001) & 0b100010001) << 3;
            new_block |= bit_3_at_bit_15;
        }
        {
            let bits_6_4 = (x >> 4) & 0x7;
            let bit_7_at_bit_14 = (x & 0x0080) << 7;

            new_block |= ((bits_6_4 * 0b1001001) & 0b100010001) << 2;
            new_block |= bit_7_at_bit_14;
        }
        {
            let bits_10_8 = (x >> 8) & 0x7;
            let bit_11_at_bit_13 = (x & 0x0800) << 2;

            new_block |= ((bits_10_8 * 0b1001001) & 0b100010001) << 1;
            new_block |= bit_11_at_bit_13;
        }
        {
            let bits_14_12 = (x >> 12) & 0x7;
            let bit_15_at_bit_12 = (x & 0x8000u16) >> 3;

            new_block |= (bits_14_12 * 0b1001001) & 0b100010001;
            new_block |= bit_15_at_bit_12;
        }
        new_block
    }

    fn rotate(&self, r: Rotation) -> Self {
        match r {
            Rotation::R0 => Self(self.0),
            Rotation::R90 => {
                // Rotating by 90 degree is the same as rotating 180 degrees then rotate 270 degrees
                Self(Self::rotate_270(self.0.reverse_bits()))
            }
            Rotation::R180 => Self(self.0.reverse_bits()),
            Rotation::R270 => Self(Self::rotate_270(self.0)),
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self(0)
    }
}

impl BitOr for Block {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Block {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl BitAnd for Block {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Block {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

/// An implementation of Grid, which decomposes the grid
/// in blocks of 4x4 elements
/// This facilitates rotations, but makes it harder to shift
#[derive(Clone)]
pub struct BlockGrid {
    /// The ℕxℕ is represented by a metagrid of 4x4 blocks.
    /// Blocks are arranged by columns. This make the extension
    /// in the X direction easy but hard in the Y direction.
    /// Shifting is easier in the X direction and harder in the Y direction.
    /// A block at meta coords (x, y) is accessible with
    /// self.grid[x * self.dim.1 + y]
    /// It is garanteed that there is at least one column
    /// with one block
    grid: Vec<Block>,

    /// The number of blocks in the X and Y direction
    dim: (usize, usize),
}

impl BlockGrid {
    fn get_block(&self, x: usize, y: usize) -> &Block {
        &self.grid[x * self.dim.1 + y]
    }

    fn get_block_mut(&mut self, x: usize, y: usize) -> &mut Block {
        &mut self.grid[x * self.dim.1 + y]
    }
}

impl Debug for BlockGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in (0..self.dim.1).rev() {
            for yy in (0..4).rev() {
                for x in 0..self.dim.0 {
                    for xx in 0..4 {
                        f.write_char(if self.get_block(x, y).get(xx, yy) {
                            'o'
                        } else {
                            '.'
                        })?;
                    }
                }
                if !(y == 0 && yy == 0) {
                    f.write_char('\n')?;
                }
            }
        }

        Ok(())
    }
}

impl Grid for BlockGrid {
    fn new() -> Self {
        // By default, a meta grid of 1x1, so a grid of 4x4
        Self {
            grid: vec![Block(0); 1],
            dim: (1, 1),
        }
    }

    fn get(&self, x: usize, y: usize) -> bool {
        let block = self.get_block(x / 4, y / 4);
        block.get(x % 4, y % 4)
    }

    fn set(&mut self, x: usize, y: usize) {
        let block = self.get_block_mut(x / 4, y / 4);
        *block = block.set(x % 4, y % 4);
    }

    fn unset(&mut self, x: usize, y: usize) {
        let block = self.get_block_mut(x / 4, y / 4);
        *block = block.unset(x % 4, y % 4);
    }

    fn count(&self) -> u32 {
        // Naive implementation, could be vectorized
        self.grid.iter().map(|b| b.count()).sum::<u32>()
    }

    fn reserve_space(&mut self, x: usize, y: usize) {
        let columns_wanted = x / 4 + if x % 4 == 0 { 0 } else { 1 };
        let rows_wanted = y / 4 + if y % 4 == 0 { 0 } else { 1 };

        // Four cases:
        // - no need to extend
        // - we only need to extend columns
        // - we only need to extend rows
        // - we must extend both rows and columns

        // Case 1: no need to extend
        if self.dim.0 >= columns_wanted && self.dim.1 >= rows_wanted {
            return;
        }
        // Case 1: we only need to add columns. Easy case
        else if self.dim.0 < columns_wanted && self.dim.1 >= rows_wanted {
            self.grid
                .resize(columns_wanted * self.dim.1, Block::default());
            self.dim.0 = columns_wanted;
        }
        // case 2: we only need to add rows. Hard case
        else if self.dim.0 >= columns_wanted && self.dim.1 < rows_wanted {
            self.grid.resize(self.dim.0 * rows_wanted, Block::default());
            // Moving columns to their new place
            for coln_index in (1..self.dim.0).rev() {
                // old column:
                let coln_start = coln_index * self.dim.1;
                let old_column = coln_start..coln_start + self.dim.1;
                // new place
                let new_place = coln_index * rows_wanted;
                self.grid.copy_within(old_column, new_place);
                // fill the empty space left behind
                self.grid[coln_start..new_place].fill(Block::default());
            }
            self.dim.1 = rows_wanted;
        }
        // We need to add both columns and rows
        else
        /* if self.dim.0 < columns_wanted && self.dim.1 < rows_wanted */
        {
            self.grid
                .resize(columns_wanted * rows_wanted, Block::default());
            // Moving columns to their new place
            for coln_index in (1..self.dim.0).rev() {
                // old column:
                let coln_start = coln_index * self.dim.1;
                let old_column = coln_start..coln_start + self.dim.1;
                // new place
                let new_place = coln_index * rows_wanted;
                self.grid.copy_within(old_column, new_place);
                // fill the empty space left behind
                self.grid[coln_start..new_place].fill(Block::default());
            }
            self.dim.0 = columns_wanted;
            self.dim.1 = rows_wanted;
        }
    }

    fn get_bounding_box(&self) -> (usize, usize) {
        let mut bb_x = 0;
        let mut bb_y = 0;

        for x in 0..self.dim.0 {
            for y in 0..self.dim.1 {
                let b = self.get_block(x, y);
                let block_bb = b.get_bounding_box();
                // If there are no elements in the block
                // do not apply max(bb, 4*(x, y)), otherwise, wrong result
                if block_bb != (0, 0) {
                    bb_x = bb_x.max(4 * x + block_bb.0);
                    bb_y = bb_y.max(4 * y + block_bb.1);
                }
            }
        }

        (bb_x, bb_y)
    }

    fn shift_x(&mut self, n: isize) {
        if n.abs() > 3 {
            panic!("Shifting by too much");
        }
        if n == 0 {
            return;
        }

        for y in 0..self.dim.1 {
            let mut right_column = 0;
            if n > 0 {
                for x in 0..self.dim.0 {
                    let block = self.get_block_mut(x, y);
                    (*block, right_column) = block.shift_x(n, right_column);
                }
            } else {
                for x in (0..self.dim.0).rev() {
                    let block = self.get_block_mut(x, y);
                    (*block, right_column) = block.shift_x(n, right_column);
                }
            };
        }
    }

    fn shift_y(&mut self, n: isize) {
        if n.abs() > 3 {
            panic!("Shifting by too much");
        }
        if n == 0 {
            return;
        }

        for x in 0..self.dim.0 {
            let mut top_row = 0u16;
            if n > 0 {
                for y in 0..self.dim.1 {
                    let block = self.get_block_mut(x, y);
                    (*block, top_row) = block.shift_y(n, top_row);
                }
            } else {
                for y in (0..self.dim.1).rev() {
                    let block = self.get_block_mut(x, y);
                    (*block, top_row) = block.shift_y(n, top_row);
                }
            };
        }
    }

    fn rotate(&self, dim: (u8, u8), r: crate::rotation::Rotation) -> Self {
        // Because of rotations & blocks taking more space than the bounding box
        // we might need to shift in -X and/or -Y direction. We do not want to shift by
        // too much, so the difference between the bounding box & the internal dimension
        // should not be more than 4x4
        assert!((self.dim.0 * 4 - dim.0 as usize) < 4);
        assert!((self.dim.1 * 4 - dim.1 as usize) < 4);

        match r {
            Rotation::R0 => self.clone(),
            Rotation::R90 => {
                let mut block_grid = BlockGrid::new();
                block_grid.reserve_space(dim.1 as usize, dim.0 as usize);

                let (x, y) = dim;
                let columns_wanted = x / 4 + if x % 4 == 0 { 0 } else { 1 };
                let rows_wanted = y / 4 + if y % 4 == 0 { 0 } else { 1 };
                let mut tmp = vec![Block::default(); (columns_wanted * rows_wanted) as usize];
                tmp.copy_from_slice(block_grid.grid.as_slice());

                // TODO: replace with ArrayChunk once released from nightly
                let mut iter = tmp.as_mut_slice().chunks_exact_mut(2);
                while let Some(chunk) = iter.next() {
                    // let (x1, x2) = Block::batch_2_rotate_90_avx2(chunk[0].0, chunk[0].0);
                    // chunk[0] = Block(x1);
                    // chunk[1] = Block(x2);
                    unsafe {
                        let (Block(x1), Block(x2)) =
                            (chunk.get_unchecked(0), chunk.get_unchecked(1));
                        let (x1, x2) = Block::batch_2_rotate_90_avx2(*x1, *x2);
                        *chunk.get_unchecked_mut(0) = Block(x1);
                        *chunk.get_unchecked_mut(1) = Block(x2);
                    }
                }
                for block in iter.into_remainder().into_iter() {
                    *block = block.rotate(Rotation::R90);
                }

                for x in 0..self.dim.0 {
                    for y in 0..self.dim.1 {
                        *block_grid.get_block_mut(self.dim.1 - 1 - y, x) = tmp[x * self.dim.1 + y];
                    }
                }

                block_grid.shift_x(dim.1 as isize - self.dim.1 as isize * 4);

                block_grid
            }
            Rotation::R180 => {
                let mut block_grid = self.clone();
                block_grid.grid.reverse();
                block_grid
                    .grid
                    .iter_mut()
                    .for_each(|block| *block = block.rotate(r));

                block_grid.shift_x(dim.0 as isize - self.dim.0 as isize * 4);
                block_grid.shift_y(dim.1 as isize - self.dim.1 as isize * 4);

                block_grid
            }
            Rotation::R270 => {
                let mut block_grid = BlockGrid::new();
                block_grid.reserve_space(dim.1 as usize, dim.0 as usize);

                for x in 0..self.dim.0 {
                    for y in 0..self.dim.1 {
                        *block_grid.get_block_mut(y, self.dim.0 - 1 - x) =
                            self.get_block(x, y).rotate(Rotation::R270);
                    }
                }

                block_grid.shift_y(dim.0 as isize - self.dim.0 as isize * 4);

                block_grid
            }
        }
    }
}

impl PartialEq for BlockGrid {
    fn eq(&self, other: &Self) -> bool {
        self.grid
            .iter()
            .zip(other.grid.iter())
            .all(|(left, right)| left == right)
    }
}

impl Eq for BlockGrid {}

impl PartialOrd for BlockGrid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

use std::cmp::Ordering;
impl Ord for BlockGrid {
    fn cmp(&self, other: &Self) -> Ordering {
        for (left, right) in self.grid.iter().zip(other.grid.iter()) {
            match left.cmp(right) {
                Ordering::Equal => (),
                ord => return ord,
            }
        }

        Ordering::Equal
    }
}

impl Hash for BlockGrid {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.grid.iter().for_each(|block| block.0.hash(state));
    }
}

impl BitOrAssign for BlockGrid {
    fn bitor_assign(&mut self, rhs: Self) {
        debug_assert_eq!(
            self.dim, rhs.dim,
            "Dimensions of the meta grid do not coincide"
        );

        for (block, block_rhs) in self.grid.iter_mut().zip(rhs.grid.iter()) {
            *block |= *block_rhs;
        }
    }
}

// _mm256_slli_epi16
// _pdep

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_get() {
        for i in 0..16 {
            assert_eq!(
                Block(1 << i).get(i / 4, i % 4),
                true,
                "testing block 0b{:0>16b} with i={i}, x={}, y={}",
                1 << i,
                i / 4,
                i % 4
            );
        }
    }

    #[test]
    fn block_set() {
        for i in 0..16 {
            assert_eq!(
                Block(0).set(i / 4, i % 4),
                Block(1 << i),
                "testing block 0b{:0>16b} with i={i}, x={}, y={}",
                1 << i,
                i / 4,
                i % 4
            );
        }
    }

    #[test]
    fn block_unset() {
        for i in 0..16 {
            assert_eq!(
                Block(0xFFFF).unset(i / 4, i % 4),
                Block(0xFFFF - (1 << i)),
                "testing block 0b{:0>16b} with i={i}, x={}, y={}",
                1 << i,
                i / 4,
                i % 4
            );
        }
    }

    #[test]
    fn block_shift_x() {
        let (mut b, mut shifted) = Block(0xF000).shift_x(-3, 0xFA50);
        assert_eq!(shifted, 0x0000);
        assert_eq!(b, Block(0xFA5F));
        (b, shifted) = b.shift_x(3, 0x0421);
        assert_eq!(shifted, 0x0FA5);
        assert_eq!(b, Block(0xF421));
        (b, shifted) = b.shift_x(-2, 0x1200);
        assert_eq!(shifted, 0x2100);
        assert_eq!(b, Block(0x12F4));
        (b, shifted) = b.shift_x(1, 0x0005);
        assert_eq!(shifted, 0x0001);
        assert_eq!(b, Block(0x2F45));
        (b, shifted) = b.shift_x(2, 0x0087);
        assert_eq!(shifted, 0x002F);
        assert_eq!(b, Block(0x4587));
        (b, shifted) = b.shift_x(-1, 0x3000);
        assert_eq!(shifted, 0x7000);
        assert_eq!(b, Block(0x3458));
        (b, shifted) = b.shift_x(0, 0xABCD);
        assert_eq!(shifted, 0x0000);
        assert_eq!(b, Block(0x3458));
    }

    #[test]
    fn block_shift_y() {
        let (mut b, mut shifted) = Block(0x8888).shift_y(-3, 0x8888);
        assert_eq!(shifted, 0x0000);
        assert_eq!(b, Block(0x9999));
        (b, shifted) = b.shift_y(3, 0x7421);
        assert_eq!(shifted, 0x4444);
        assert_eq!(b, Block(0xFCA9));
        (b, shifted) = b.shift_y(-2, 0x0040);
        assert_eq!(shifted, 0xC084);
        assert_eq!(b, Block(0x3362));
        (b, shifted) = b.shift_y(1, 0x0100);
        assert_eq!(shifted, 0x0000);
        assert_eq!(b, Block(0x67C4));
        (b, shifted) = b.shift_y(2, 0x0023);
        assert_eq!(shifted, 0x1131);
        assert_eq!(b, Block(0x8C23));
        (b, shifted) = b.shift_y(-1, 0x0080);
        assert_eq!(shifted, 0x0008);
        assert_eq!(b, Block(0x4691));
        (b, shifted) = b.shift_y(0, 0xABCD);
        assert_eq!(shifted, 0x0000);
        assert_eq!(b, Block(0x4691));
    }

    #[test]
    fn block_rotate_270() {
        assert_eq!(Block(0xA88E).rotate(Rotation::R270), Block(0xF890));
        assert_eq!(Block(0x0611).rotate(Rotation::R270), Block(0x022C));
        assert_eq!(Block(0x5160).rotate(Rotation::R270), Block(0x0543));
    }

    #[test]
    fn block_rotate_2x90_is_180() {
        for i in 0..u16::MAX {
            let b = Block(i);
            assert_eq!(
                b.rotate(Rotation::R90).rotate(Rotation::R90),
                b.rotate(Rotation::R180),
                "testing double rotation of block {i} (90° rotation {:?})",
                b.rotate(Rotation::R90)
            );
        }
    }

    #[test]
    fn block_rotate_3x90_is_270() {
        for i in 0..u16::MAX {
            let b = Block(i);
            assert_eq!(
                b.rotate(Rotation::R90)
                    .rotate(Rotation::R90)
                    .rotate(Rotation::R90),
                b.rotate(Rotation::R270),
                "testing triple rotation of block {i} (90° rotation {:?}) (180° rotation {:?})",
                b.rotate(Rotation::R90),
                b.rotate(Rotation::R90).rotate(Rotation::R90),
            );
        }
    }

    #[test]
    fn block_rotate_4x90_is_0() {
        for i in 0..u16::MAX {
            let b = Block(i);
            assert_eq!(
                b.rotate(Rotation::R90)
                    .rotate(Rotation::R90)
                    .rotate(Rotation::R90)
                    .rotate(Rotation::R90),
                b,
                "testing four rotation of block {i}",
            );
        }
    }
}
