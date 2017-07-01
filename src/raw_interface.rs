use sc::platform::nr::{USERFAULTFD, IOCTL, READ};
use std::os::unix::io::RawFd;
use std::os::raw::c_void;
use std::io::{Error,ErrorKind};

pub mod defines {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
    // This is necessary to work around a bug in bindgen with functional macros
    // https://github.com/servo/rust-bindgen/issues/753
    pub const UFFD_API: u64 = _T_UFFD_API as u64;
    pub const UFFDIO_API: u64 = _T_UFFDIO_API as u64;
    pub const UFFDIO_REGISTER: u64 = _T_UFFDIO_REGISTER as u64;
    pub const UFFDIO_UNREGISTER: u64 = _T_UFFDIO_UNREGISTER as u64;
    pub const UFFDIO_WAKE: u64 = _T_UFFDIO_WAKE as u64;
    pub const UFFDIO_COPY: u64 = _T_UFFDIO_COPY as u64;
    pub const UFFDIO_ZEROPAGE: u64 = _T_UFFDIO_ZEROPAGE as u64;
    pub const UFFDIO_REGISTER_MODE_MISSING: u64 = _T_UFFDIO_REGISTER_MODE_MISSING as u64;
    pub const UFFDIO_REGISTER_MODE_WP: u64 = _T_UFFDIO_REGISTER_MODE_WP as u64;
    pub const UFFDIO_COPY_MODE_DONTWAKE: u64 = _T_UFFDIO_COPY_MODE_DONTWAKE as u64;
    pub const UFFDIO_ZEROPAGE_MODE_DONTWAKE: u64 = _T_UFFDIO_ZEROPAGE_MODE_DONTWAKE as u64;
    pub const UFFD_API_IOCTLS: u64 = _T_UFFD_API_IOCTLS as u64;
    pub const UFFD_API_RANGE_IOCTLS: u64 = _T_UFFD_API_RANGE_IOCTLS as u64;
    pub const UFFD_API_RANGE_IOCTLS_BASIC: u64 = _T_UFFD_API_RANGE_IOCTLS_BASIC as u64;
}

impl From<super::Range> for defines::uffdio_range {
    fn from(other: super::Range) -> defines::uffdio_range {
        defines::uffdio_range {
            start: other.start as u64,
            len: other.len as u64
        }
    }
}

pub fn userfaultfd(flags: usize) -> RawFd {
    unsafe { syscall!(USERFAULTFD, flags) as i32 }
}
fn ioctl(fd: RawFd, cmd: u64, arg: *mut c_void) -> Result<i64, Error> {
    unsafe { retry(||syscall!(IOCTL, fd, cmd, arg) as i64) }
}
pub fn read(fd: RawFd, buf: &mut [u8]) -> Result<usize, Error> {
    unsafe { retry(||syscall!(READ, fd, buf as *mut _ as *mut c_void, buf.len()) as i64).map(|x| x as usize) }
}
fn cvt(v: i64) -> Result<i64, Error> {
    if v < 0 {
        Err(Error::from_raw_os_error(-v as i32))
    } else {
        Ok(v)
    }
}
fn retry<F: FnMut() -> i64>(mut f: F) -> Result<i64, Error> {
    loop {
        match cvt(f()) {
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
            x => return x
        }
    }
}
pub fn uffdio_api(fd: RawFd, t: &mut defines::uffdio_api) -> Result<(), Error> {
    match ioctl(fd, defines::UFFDIO_API, t as *mut _ as *mut c_void) {
        Err(e) => Err(e),
        Ok(0) => Ok(()),
        Ok(x) => panic!("Unexpected return value from UFFDIO_API ioctl: {}", x)
    }
}
pub fn uffdio_register(fd: RawFd, mode: u64, range: defines::uffdio_range) -> Result<u64, Error> {
    let mut t = defines::uffdio_register {
        range: range,
        mode: mode,
        ioctls: 0
    };
    match ioctl(fd, defines::UFFDIO_REGISTER, &mut t as *mut _ as *mut c_void) {
        Err(e) => Err(e),
        Ok(0) => Ok(t.ioctls),
        Ok(x) => panic!("Unexpected return value from UFFDIO_REGISTER ioctl: {}", x)
    }
}
pub fn uffdio_unregister(fd: RawFd, mut range: defines::uffdio_range) -> Result<(), Error> {
    match ioctl(fd, defines::UFFDIO_UNREGISTER, &mut range as *mut _ as *mut c_void) {
        Err(e) => Err(e),
        Ok(0) => Ok(()),
        Ok(x) => panic!("Unexpected return value from UFFDIO_UNREGISTER ioctl: {}", x)
    }
}
pub fn uffdio_copy(fd: RawFd, mut copy: defines::uffdio_copy) -> Result<(), Error> {
    loop {
        match ioctl(fd, defines::UFFDIO_COPY, &mut copy as *mut _ as *mut c_void) {
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                copy.dst += copy.copy as u64;
                copy.src += copy.copy as u64;
                copy.len -= copy.copy as u64;
            }
            Err(e) => return Err(e),
            Ok(0) => return Ok(()),
            Ok(x) => panic!("Unexpected return value from UFFDIO_COPY ioctl: {}", x)
        }
    }
}
pub fn uffdio_zeropage(fd: RawFd, mut zeropage: defines::uffdio_zeropage) -> Result<(), Error> {
    println!("{:?}", zeropage);
    loop {
        match ioctl(fd, defines::UFFDIO_ZEROPAGE, &mut zeropage as *mut _ as *mut c_void) {
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                zeropage.range.start += zeropage.zeropage as u64;
                zeropage.range.len -= zeropage.zeropage as u64;
            }
            Err(e) => return Err(e),
            Ok(0) => return Ok(()),
            Ok(x) => panic!("Unexpected return value from UFFDIO_COPY ioctl: {}", x)
        }
    }
}
pub fn uffdio_wake(fd: RawFd, mut range: defines::uffdio_range) -> Result<(), Error> {
    match ioctl(fd, defines::UFFDIO_WAKE, &mut range as *mut _ as *mut c_void) {
        Err(e) => Err(e),
        Ok(0) => Ok(()),
        Ok(x) => panic!("Unexpected return value from UFFDIO_WAKE ioctl: {}", x)
    }
}
