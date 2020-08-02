extern crate v4l;

use v4l::device::List;

fn main() {
    let list = List::new();

    for dev in list {
        println!("{}: {}", dev.index().unwrap(), dev.name().unwrap());
    }
}
