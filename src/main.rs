use soc::SoC;

mod soc;

#[allow(non_snake_case)]
mod cpu;

fn main() {
    let mut soc = SoC::new();
    soc.tick();
}
