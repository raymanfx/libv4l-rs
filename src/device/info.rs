use std::path::{Path, PathBuf};
use std::{fs, io, mem};

use crate::v4l_sys::*;
use crate::Capabilities;
use crate::{ioctl, v4l2};

/// Represents a video4linux device node
pub struct DeviceInfo {
    /// File descriptor
    fd: std::os::raw::c_int,
    /// Device node path
    path: PathBuf,
}

impl Drop for DeviceInfo {
    fn drop(&mut self) {
        v4l2::close(self.fd).unwrap();
    }
}

impl DeviceInfo {
    /// Returns a device node observer by path
    ///
    /// The device is opened in read only mode.
    ///
    /// # Arguments
    ///
    /// * `path` - Node path (usually a character device or sysfs entry)
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::DeviceInfo;
    /// let node = DeviceInfo::new("/dev/video0");
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        let fd = v4l2::open(path, libc::O_RDONLY)?;

        Ok(DeviceInfo {
            fd,
            path: PathBuf::from(path),
        })
    }

    /// Returns the absolute path of the device node
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the index of the device node
    pub fn index(&self) -> Option<usize> {
        let file_name = self.path.file_name()?;

        let mut index_str = String::new();
        for c in file_name
            .to_str()?
            .chars()
            .rev()
            .collect::<String>()
            .chars()
        {
            if !c.is_digit(10) {
                break;
            }

            index_str.push(c);
        }

        let index = index_str.parse::<usize>();
        if index.is_err() {
            return None;
        }

        Some(index.unwrap())
    }

    /// Returns name of the device by parsing its sysfs entry
    pub fn name(&self) -> Option<String> {
        let index = self.index()?;
        let path = format!("{}{}{}", "/sys/class/video4linux/video", index, "/name");
        let name = fs::read_to_string(path);
        match name {
            Ok(name) => Some(name),
            Err(_) => None,
        }
    }

    /// Query for device capabilities
    ///
    /// This returns video4linux framework defined information such as card, driver, etc.
    pub fn query_caps(&self) -> io::Result<Capabilities> {
        unsafe {
            let mut v4l2_caps: v4l2_capability = mem::zeroed();
            v4l2::ioctl(
                self.fd,
                ioctl::codes::VIDIOC_QUERYCAP,
                &mut v4l2_caps as *mut _ as *mut std::os::raw::c_void,
            )?;

            Ok(Capabilities::from(v4l2_caps))
        }
    }
}

/// Represents an iterable list of valid devices
#[derive(Default)]
pub struct DeviceList {
    /// Position in the list
    pos: usize,
    /// All paths representing potential video4linux devices
    paths: Vec<PathBuf>,
}

impl DeviceList {
    /// Returns a list of devices currently known to the system
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::DeviceList;
    /// let list = DeviceList::new();
    /// for dev in list {
    ///     print!("{}{}", dev.index().unwrap(), dev.name().unwrap());
    /// }
    /// ```
    pub fn new() -> Self {
        let mut list = DeviceList {
            pos: 0,
            paths: Vec::new(),
        };

        let nodes = fs::read_dir("/dev");
        if let Ok(nodes) = nodes {
            for node in nodes {
                if node.is_err() {
                    continue;
                }
                let node = node.unwrap();
                let file_name = node.file_name();
                let file_name = file_name.to_str().unwrap();

                if file_name.starts_with("video") {
                    list.paths.push(node.path());
                }
            }
        }

        list.paths.sort();
        list
    }
}

impl Iterator for DeviceList {
    type Item = DeviceInfo;

    fn next(&mut self) -> Option<DeviceInfo> {
        let pos = self.pos;
        if pos == self.paths.len() {
            return None;
        }

        self.pos += 1;
        let dev = DeviceInfo::new(&self.paths[pos]);
        match dev {
            Ok(dev) => Some(dev),
            Err(_) => None,
        }
    }
}
