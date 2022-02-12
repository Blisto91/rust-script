extern crate cli_setup;
use cli_setup::setup_manpages;

use std::fs;

pub const MAN_PAGE: &str = include_str!("man/rust-script.1");

fn main() {
    setup_manpages(MAN_PAGE, "rust-script");
}
