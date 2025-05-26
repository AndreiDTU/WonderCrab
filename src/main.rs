use soc::SoC;

pub mod soc;
pub mod bus;
pub mod dma;

#[allow(non_snake_case)]
pub mod cpu;

fn main() {
    let mut soc = SoC::new();
    soc.tick();
}

#[macro_export]
macro_rules! assert_eq_hex {
    ($left:expr, $right:expr) => {
        let left_val = $left;
        let right_val = $right;
        assert!(
            left_val == right_val,
            "assertion `left == right` failed\n  left: 0x{:X}\n right: 0x{:X}",
            left_val, right_val,
        )
    };
}
