mod driver;
mod protocol;

use crate::driver::*;

fn main() {
    let mut was_enabled = false;
    set_system_environment_privilege(true, &mut was_enabled);
    println!("{}", was_enabled);
    println!("{}", get_base_address(4));
    println!("{}", test());
}
