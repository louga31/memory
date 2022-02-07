use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=../protocol.rs");

    let destination = "src";
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let mut paths_to_copy = Vec::new();
    paths_to_copy.push("../protocol.rs");
    copy_items(&paths_to_copy, destination, &copy_options)?;

    Ok(())
}
