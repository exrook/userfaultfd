#[macro_use]
extern crate sc;
use std::os::unix::io::{AsRawFd,FromRawFd,IntoRawFd,RawFd};

mod raw_interface {
    use sc::platform::nr::{USERFAULTFD, IOCTL};
    use std::os::unix::io::{AsRawFd,FromRawFd,IntoRawFd,RawFd};
    use std::os::raw::{c_int, c_void};
    use std::io::{Error,ErrorKind};

    pub mod defines {
        include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
    }

    pub fn userfaultfd(flags: usize) -> RawFd {
        unsafe { syscall!(USERFAULTFD, flags) as i32 }
    }
    fn ioctl(fd: RawFd, cmd: i64, arg: *mut c_void) -> i64 {
        unsafe { syscall!(IOCTL, fd, cmd, arg) as i64}
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
        match retry(||ioctl(fd, defines::UFFDIO_API, t as *mut _ as *mut c_void)) {
            Err(e) => Err(e),
            Ok(0) => Ok(()),
            Ok(x) => panic!("Unexpected return value from UFFDIO_API ioctl: {}", x)
        }
    }
    pub fn uffdio_register(fd: RawFd, mode: u64, start: u64, len: u64) -> Result<u64, Error> {
        let mut t = defines::uffdio_register {
            range: defines::uffdio_range {
                start: start,
                len: len,
            },
            mode: mode,
            ioctls: 0
        };
        match retry(||ioctl(fd, defines::UFFDIO_REGISTER, &mut t as *mut _ as *mut c_void)) {
            Err(e) => Err(e),
            Ok(0) => Ok(t.ioctls),
            Ok(x) => panic!("Unexpected return value from UFFDIO_REGISTER ioctl: {}", x)
        }
    }
    pub fn uffdio_unregister(fd: RawFd, mode: u64, start: u64, len: u64) -> Result<(), Error> {
        let mut t = defines::uffdio_range {
            start: start,
            len: len,
        };
        match retry(||ioctl(fd, defines::UFFDIO_UNREGISTER, &mut t as *mut _ as *mut c_void)) {
            Err(e) => Err(e),
            Ok(0) => Ok(()),
            Ok(x) => panic!("Unexpected return value from UFFDIO_REGISTER ioctl: {}", x)
        }
    }
    pub fn uffdio_copy(fd: RawFd, mode: u64, start: u64, len: u64) -> Result<(), Error> {
        let mut t = defines::uffdio_range {
            start: start,
            len: len,
        };
        match retry(||ioctl(fd, defines::UFFDIO_UNREGISTER, &mut t as *mut _ as *mut c_void)) {
            Err(e) => Err(e),
            Ok(0) => Ok(()),
            Ok(x) => panic!("Unexpected return value from UFFDIO_UNREGISTER ioctl: {}", x)
        }
    }
}

#[cfg(test)]
mod tests {
    use ::raw_interface;
    #[test]
    fn it_works() {
        let fd = raw_interface::userfaultfd(0);
        println!("fd: {:?}", fd);
        let mut t = raw_interface::defines::uffdio_api {
            api: raw_interface::defines::UFFD_API,
            features: 0,
            ioctls: 0
        };
        let res = raw_interface::uffdio_api(fd, &mut t);
        println!("res: {:?} t: {:?}", res, t);
    }
}

pub struct UFFDHandle(RawFd);

impl UFFDHandle {
    fn create(flags: usize) -> UFFDHandle {
        UFFDHandle(raw_interface::userfaultfd(flags))
    }
}

impl AsRawFd for UFFDHandle {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl FromRawFd for UFFDHandle {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        UFFDHandle(fd)
    }
}

impl IntoRawFd for UFFDHandle {
    fn into_raw_fd(self) -> RawFd {
        self.as_raw_fd()
    }
}
