use polyominos::{board::Board, database::*, polyomino::*};

const LIMIT: u8 = 4;

fn main() {
    let p1 = Polyomino::trivial();
    for p in decline(&p1).into_iter().take(1) {
        println!("Declinaison:");
        println!("{p:?}");
        let smallest = smallest_rotation(p);
        println!("Smallest found:");
        println!("{smallest:?}");
    }

    // for i in 0..32 {
    //     let mut x = Board::new();
    //     for j in 0..32 {
    //         x.set(i, j);
    //     }

    //     // println!("X:");
    //     // println!("{:?}", Polyomino::from((32, 32), x, Board::new()));

    //     let y = x.rotate((32, 32), &polyominos::rotation::Rotation::R90);

    //     // println!("Y:");
    //     println!("{:?}", Polyomino::from((32, 32), y, Board::new()));
    // }
    // let mut db = Database::new();

    // loop {
    //     let p = {
    //         match db.pop() {
    //             None => {
    //                 db.flush();
    //                 continue;
    //             }
    //             Some(p) => p,
    //         }
    //     };

    //     if p.square_count > LIMIT {
    //         break;
    //     }

    //     let declinaison = decline(&p);

    //     for p in declinaison.into_iter() {
    //         let smallest = smallest_rotation(p);

    //         // println!("Smallest found:");
    //         // println!("{smallest:?}");

    //         db.register(smallest);
    //     }
    // }

    // for (i, (cnt, stat)) in db.counts().zip(db.stats()).enumerate() {
    //     let squares = i + 1;
    //     let redundant = stat - cnt;
    //     println!("With {squares} squares: {cnt} ({redundant} redundancies)")
    // }
}

// NOTES:
//
// When adding a square, the dimension can either increase in the x direction,
// or in the y direction, or not increase (if the square is added in a crease
// or a hole for example)
// This can be used to split databases between sizes, to spread memory usage across
// different nodes
