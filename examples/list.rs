extern crate v4l;

use v4l::DeviceList;

fn main() {
    let list = DeviceList::new();

    for dev in list {
        println!("{}: {}", dev.index().unwrap(), dev.name().unwrap());

        let caps = dev.query_caps();
        match caps {
            Ok(caps) => println!("{}", caps),
            Err(e) => println!("{}", e),
        }
    }
}
