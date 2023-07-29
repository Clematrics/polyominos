use polyominos::{
    database::*,
    grid::{are_equal, transfer, Grid},
    grids::{block_grid::BlockGrid, naive::Naive},
    polyomino::*,
};

const LIMIT: u8 = 12;

fn main() {
    let mut db = Database::<BlockGrid>::new();

    loop {
        let p = {
            match db.pop() {
                None => {
                    db.flush();
                    continue;
                }
                Some(p) => p,
            }
        };

        if p.square_count >= LIMIT {
            break;
        }

        let declinaison = decline(&p);

        for p in declinaison.into_iter() {
            println!("Searching smallest rotation of\n{:?}", p);
            // println!("p repr:\n{:?}", p.repr);
            // println!("p mask:\n{:?}", p.mask);

            let witness = Polyomino::<Naive> {
                square_count: p.square_count,
                dimension: p.dimension,
                repr: transfer(&p.repr),
                mask: transfer(&p.mask),
            };

            let (smallest, r) = smallest_rotation(p);

            are_equal(&smallest.repr, &witness.repr.rotate(witness.dimension, r));
            are_equal(&smallest.mask, &witness.mask.rotate(witness.dimension, r));

            println!("Smallest found:");
            println!("{smallest:?}");

            db.register(smallest);
        }
    }

    for (i, (cnt, stat)) in db.counts().zip(db.stats()).enumerate() {
        let squares = i + 1;
        let redundant = stat - cnt;
        println!("With {squares} squares: {cnt} ({redundant} redundancies)")
    }
}

// NOTES:
//
// When adding a square, the dimension can either increase in the x direction,
// or in the y direction, or not increase (if the square is added in a crease
// or a hole for example)
// This can be used to split databases between sizes, to spread memory usage across
// different nodes
