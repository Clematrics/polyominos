use std::ops::BitOrAssign;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use polyominos::database::Database;
use polyominos::grid::Grid;
use polyominos::grids::{block_grid::BlockGrid, naive::Naive};
use polyominos::polyomino;

fn polyominos_of_count<T>(square_count: usize) -> Vec<(u8, u8, T)>
where
    T: Grid + BitOrAssign,
{
    let mut db = Database::<T>::new();

    // for _ in 1..square_count {

    // }
    let mut square_count_in_queue = 0;
    loop {
        let p = {
            match db.pop() {
                None => {
                    db.flush();
                    square_count_in_queue += 1;
                    if square_count_in_queue == square_count {
                        break;
                    }
                    continue;
                }
                Some(p) => p,
            }
        };
        let declinaison = polyomino::decline(&p);

        for p in declinaison.into_iter() {
            let (smallest, _r) = polyomino::smallest_rotation(p);
            db.register(smallest);
        }
    }

    db.to_queue()
        .into_iter()
        .map(|p| (p.dimension.0, p.dimension.1, p.repr))
        .collect()
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let vec_naive = polyominos_of_count::<Naive>(10);
    let vec_block = polyominos_of_count::<BlockGrid>(10);

    println!("Vec count: {}", vec_naive.len());

    c.bench_function("Naive count 10 rotation 90", |b| {
        b.iter_with_large_drop(|| {
            vec_naive
                .iter()
                .map(|(sx, sy, grid)| grid.rotate((*sx, *sy), polyominos::rotation::Rotation::R90))
                .collect::<Vec<Naive>>()
        })
    });
    c.bench_function("BlockGrid count 10 rotation 90", |b| {
        b.iter_with_large_drop(|| {
            vec_block
                .iter()
                .map(|(sx, sy, grid)| grid.rotate((*sx, *sy), polyominos::rotation::Rotation::R90))
                .collect::<Vec<BlockGrid>>()
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
