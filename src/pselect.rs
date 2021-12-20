use std::os::unix::io::RawFd;
use std::{io, mem, ptr, time};

#[derive(Clone, Copy)]
pub struct FdSet(libc::fd_set);

impl FdSet {
    pub fn new() -> FdSet {
        unsafe {
            let mut raw_fd_set = mem::MaybeUninit::<libc::fd_set>::uninit();
            libc::FD_ZERO(raw_fd_set.as_mut_ptr());
            FdSet(raw_fd_set.assume_init())
        }
    }

    pub fn set(&mut self, fd: RawFd) {
        unsafe {
            libc::FD_SET(fd, &mut self.0);
        }
    }
}

fn to_fdset_ptr(opt: Option<&mut FdSet>) -> *mut libc::fd_set {
    match opt {
        None => ptr::null_mut(),
        Some(&mut FdSet(ref mut raw_fd_set)) => raw_fd_set,
    }
}
fn to_ptr<T>(opt: Option<&T>) -> *const T {
    match opt {
        None => ptr::null::<T>(),
        Some(p) => p,
    }
}

pub fn pselect(
    nfds: libc::c_int,
    readfds: Option<&mut FdSet>,
    writefds: Option<&mut FdSet>,
    errorfds: Option<&mut FdSet>,
    timeout: Option<&libc::timespec>,
    sigmask: Option<&libc::sigset_t>,
) -> io::Result<usize> {
    match unsafe {
        libc::pselect(
            nfds,
            to_fdset_ptr(readfds),
            to_fdset_ptr(writefds),
            to_fdset_ptr(errorfds),
            to_ptr(timeout),
            to_ptr(sigmask),
        )
    } {
        -1 => Err(io::Error::last_os_error()),
        res => Ok(res as usize),
    }
}

pub fn make_timespec(duration: time::Duration) -> libc::timespec {
    libc::timespec {
        tv_sec: duration.as_secs() as i64,
        tv_nsec: duration.subsec_nanos() as i64,
    }
}
