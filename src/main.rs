use polyominos::{database::*, polyomino::*};

const LIMIT: u8 = 10;

fn main() {
    let mut db = Database::new();

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

        if p.square_count > LIMIT {
            break;
        }

        let declinaison = decline(&p);

        for p in declinaison.into_iter() {
            let smallest = smallest_rotation(p);

            // println!("Smallest found:");
            // println!("{smallest:?}");

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
