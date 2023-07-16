use std::collections::VecDeque;

use polyominos::{board::*, database::*, polyomino::*};

const LIMIT: u8 = 10;

fn main() {
    let mut db = Database(vec![]);

    let repr = {
        let mut arr = [[false; SIZE]; SIZE];
        arr[1][1] = true;
        Board(arr)
    };
    let mask = {
        let mut arr = [[false; SIZE]; SIZE];
        arr[1][0] = true;
        arr[0][1] = true;
        arr[1][2] = true;
        arr[2][1] = true;
        Board(arr)
    };
    let trivial_polyomino = Polyomino {
        square_count: 1,
        dimension: (3, 3),
        repr,
        mask,
    };
    db.add_or_reject(&trivial_polyomino);

    let mut queue: VecDeque<_> = [trivial_polyomino].into();

    loop {
        let p = queue.pop_front().unwrap();

        if p.square_count > LIMIT {
            break;
        }

        let declinaison = decline(&p);

        for p in declinaison.into_iter() {
            let smallest = smallest_rotation(p);

            // println!("Smallest found:");
            // println!("{smallest:?}");

            if db.add_or_reject(&smallest) {
                // println!("Not seen before");
                queue.push_back(smallest)
            }
        }
    }

    for (i, map) in db.0.iter().enumerate() {
        let cnt: u128 = map.iter().map(|(_, set)| set.len() as u128).sum();

        let squares = i + 1;
        println!("With {squares} squares: {cnt}")
    }
}
