extern crate v4l;

use v4l::DeviceList;

fn main() {
    let list = DeviceList::new();

    for dev in list {
        println!("{}: {}", dev.index().unwrap(), dev.name().unwrap());
    }
}
