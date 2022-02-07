mod driver;
mod protocol;

use crate::driver::*;

fn main() {
    println!("Hello, world!");
    println!("{}", get_base_address(4));
}
