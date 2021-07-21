use v4l::context;

fn main() {
    let devices = context::enum_devices();

    for dev in devices {
        println!("{}: {}", dev.index(), dev.name().unwrap());
    }
}
