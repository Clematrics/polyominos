use crate::grid::Grid;

use super::naive::Naive;

use paste::paste;
macro_rules! test {
    ($ty:expr) => {
        paste! {
            #[test]
            #[allow(non_snake_case)]
            fn [<test_ $ty>]() {
                test_grid::<$ty>();
            }
        }
    };
}

test!(Naive);

fn test_grid<T>()
where
    T: Grid,
{
    // Do stuff
}
