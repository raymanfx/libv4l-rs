use std::fs;

use crate::device::Node;

/// Returns a list of devices currently known to the system
///
/// # Example
///
/// ```
/// use v4l::context;
/// for dev in context::enum_devices() {
///     print!("{}{}", dev.index(), dev.name().unwrap());
/// }
/// ```
pub fn enum_devices() -> Vec<Node> {
    let mut devices = Vec::new();

    let entries = fs::read_dir("/dev");
    if let Ok(entries) = entries {
        for dentry in entries {
            let dentry = match dentry {
                Ok(dentry) => dentry,
                Err(_) => continue,
            };

            let file_name = dentry.file_name();
            let file_name = file_name.to_str().unwrap();

            if file_name.starts_with("video") {
                let node = Node::new(dentry.path());
                devices.push(node);
            }
        }
    }

    devices
}
