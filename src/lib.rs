#[macro_use]
extern crate sc;
#[macro_use]
extern crate bitflags;
#[cfg(feature = "mio")]
extern crate mio;

#[cfg(feature = "mio")]
use mio::unix::EventedFd;

use std::os::unix::io::{AsRawFd,FromRawFd,IntoRawFd,RawFd};
use std::io::Error;
use std::io;

mod raw_interface;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Builder {
    close_on_exec: bool,
    non_block: bool,
    event_fork: bool,
    event_remap: bool,
    event_remove: bool,
    event_unmap: bool,
    hugetlbfs: bool,
    shmem: bool,
}

macro_rules! builder_methods {
    ( $($arg:ident : $type:ty),+) => {
        $(pub fn $arg(mut self, value: $type) -> Self {
            self.$arg = value;
            self
        })+
    }
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    builder_methods!{
        close_on_exec: bool,
        non_block: bool,
        event_fork: bool,
        event_remap: bool,
        event_remove: bool,
        event_unmap: bool,
        hugetlbfs: bool,
        shmem: bool
    }

    pub fn create(self) -> Result<(Handle, u64), Error> {
        let flags = 
            if self.close_on_exec { raw_interface::defines::O_CLOEXEC }  else { 0 } |
            if self.non_block     { raw_interface::defines::O_NONBLOCK } else { 0 };
        let raw_fd = raw_interface::userfaultfd(flags as usize);

        let features =
              if self.event_fork   { raw_interface::defines::UFFD_FEATURE_EVENT_FORK        } else { 0 }
            | if self.event_remap  { raw_interface::defines::UFFD_FEATURE_EVENT_REMAP       } else { 0 }
            | if self.event_remove { raw_interface::defines::UFFD_FEATURE_EVENT_REMOVE      } else { 0 }
            | if self.event_unmap  { raw_interface::defines::UFFD_FEATURE_EVENT_UNMAP       } else { 0 }
            | if self.hugetlbfs    { raw_interface::defines::UFFD_FEATURE_MISSING_HUGETLBFS } else { 0 }
            | if self.shmem        { raw_interface::defines::UFFD_FEATURE_MISSING_SHMEM     } else { 0 };

        let mut req = raw_interface::defines::uffdio_api {
            api: raw_interface::defines::UFFD_API,
            features: features as u64,
            ioctls: 0
        };
        raw_interface::uffdio_api(raw_fd, &mut req).map(|()|(Handle(raw_fd), req.ioctls))
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            close_on_exec: false,
            non_block: false,
            event_fork: false,
            event_remap: false,
            event_remove: false,
            event_unmap: false,
            hugetlbfs: false,
            shmem: false,
        }
    }
}

#[derive(Debug)]
pub struct Handle(RawFd);

pub struct Range {
    pub start: *mut u8,
    pub len: usize
}

impl<'a,T> From<&'a mut [T]> for Range {
    fn from(s: &'a mut [T]) -> Self {
        Self {
            start: s.as_mut_ptr() as *mut u8,
            len: s.len() * std::mem::size_of::<T>()
        }
    }
}

impl<'a> From<&'a PagefaultMessage> for Range {
    fn from(msg: &'a PagefaultMessage) -> Self {
        Self {
            start: msg.address as *mut u8,
            len: 4096
        }
    }
}

bitflags! {
    pub struct RegisterMode: u64 {
        const REGISTER_MISSING = raw_interface::defines::UFFDIO_REGISTER_MODE_MISSING;
        const REGISTER_WP = raw_interface::defines::UFFDIO_REGISTER_MODE_WP;
    }
}

bitflags! {
    pub struct CopyMode: u64 {
        const COPY_DONTWAKE = raw_interface::defines::UFFDIO_COPY_MODE_DONTWAKE;
    }
}

bitflags! {
    pub struct ZeropageMode: u64 {
        const ZEROPAGE_DONTWAKE = raw_interface::defines::UFFDIO_ZEROPAGE_MODE_DONTWAKE;
    }
}

bitflags! {
    pub struct Ioctls: u64 {
        const IOCTL_API = 1 << raw_interface::defines::_UFFDIO_API;
        const IOCTL_REGISTER = 1 << raw_interface::defines::_UFFDIO_REGISTER;
        const IOCTL_UNREGISTER = 1 << raw_interface::defines::_UFFDIO_UNREGISTER;
        const IOCTL_API_IOCTLS = raw_interface::defines::UFFD_API_IOCTLS;
        const IOCTL_WAKE = 1 << raw_interface::defines::_UFFDIO_WAKE;
        const IOCTL_COPY = 1 << raw_interface::defines::_UFFDIO_COPY;
        const IOCTL_RANGE_IOCTLS = raw_interface::defines::UFFD_API_RANGE_IOCTLS;
        const IOCTL_ZEROPAGE = 1 << raw_interface::defines::_UFFDIO_ZEROPAGE;
        const IOCTL_RANGE_IOCTLS_BASIC = raw_interface::defines::UFFD_API_RANGE_IOCTLS_BASIC;
    }
}

impl std::fmt::Display for Ioctls {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        if self.contains(IOCTL_API) {
            write!(f, "IOCTL_API")?;
        }
        if self.contains(IOCTL_REGISTER) {
            write!(f, "IOCTL_REGISTER")?;
        }
        if self.contains(IOCTL_UNREGISTER) {
            write!(f, "IOCTL_UNREGISTER")?;
        }
        if self.contains(IOCTL_WAKE) {
            write!(f, "IOCTL_WAKE")?;
        }
        if self.contains(IOCTL_COPY) {
            write!(f, "IOCTL_COPY")?;
        }
        if self.contains(IOCTL_ZEROPAGE) {
            write!(f, "IOCTL_ZEROPAGE")?;
        }
        write!(f, "]")
    }
}
impl Handle {
    /// `(Since Linux 4.3.)` Register a memory address range with the userfaultfd object. The pages in the
    /// range must be "compatible".
    /// 
    /// Up to Linux kernel 4.11, only private anonymous ranges are compatible for registering with
    /// `register()`
    /// 
    /// Since Linux 4.11, hugetlbfs and shared memory ranges are also compatible with `register()`.
    /// 
    /// The `range` field defines a memory range starting at `start` and continuing for `len` bytes that should be
    /// handled by the `userfaultfd`.
    /// 
    /// The `mode` field defines the mode of operation desired for this memory region. The following values may
    /// be bitwise ORed to set the userfaultfd mode for the specified range:
    /// 
    /// * `REGISTER_MISSING`
    ///        Track page faults on missing pages.
    /// 
    /// * `REGISTER_WP`
    ///        Track page faults on write-protected pages.
    /// 
    /// Currently, the only supported mode is `REGISTER_MISSING`.
    /// 
    /// If the operation is successful, the kernel returns which operations are available for the specified
    /// range.
    ///
    /// Possible errors include:
    /// 
    /// * `EBUSY`  A mapping in the specified range is registered with another userfaultfd object.
    /// 
    /// * `EFAULT` argp refers to an address that is outside the calling process's accessible address space.
    /// 
    /// * `EINVAL` An invalid or unsupported bit was specified in the mode field; or the mode field was zero.
    /// 
    /// * `EINVAL` There is no mapping in the specified address range.
    /// 
    /// * `EINVAL` range.start  or  range.len is not a multiple of the system page size; or, range.len is zero; or
    ///        these fields are otherwise invalid.
    /// 
    /// * `EINVAL` There as an incompatible mapping in the specified address range.

    pub fn register<T: Into<Range>>(&self, range: T, mode: RegisterMode) -> Result<Ioctls, Error> {
        raw_interface::uffdio_register(self.0, mode.bits(), range.into().into()).map(|x|Ioctls::from_bits_truncate(x))
    }
    /// `(Since Linux 4.3.)` Unregister a memory address range from userfaultfd. The pages in the range must
    /// be "compatible" (see the description of `register`.)
    /// 
    /// The address range to unregister is specified in the `Range` structure.
    /// 
    /// Possible errors include:
    /// 
    /// * `EINVAL` Either the start or the len field of the ufdio_range structure was not a multiple of the system
    ///        page size; or the len field was zero; or these fields were otherwise invalid.
    /// 
    /// * `EINVAL` There as an incompatible mapping in the specified address range.
    /// 
    /// * `EINVAL` There was no mapping in the specified address range.
    pub fn unregister<T: Into<Range>>(&self, range: T) -> Result<(), Error> {
        raw_interface::uffdio_unregister(self.0, range.into().into())
    }
    /// `(Since  Linux 4.3.)` Atomically copy a continuous memory chunk into the userfault registered range and
    /// optionally wake up the blocked thread. The source and destination addresses and the number of bytes
    /// to copy are specified by the `src`, `dst`, and `len` fields:
    ///
    ///
    /// ```rust,ignore
    /// fn copy( &self,
    ///     dst: *mut u8,       /* Source of copy (sic) */
    ///     src: *mut u8,       /* Destination of copy */
    ///     len: u64,       /* Number of bytes to copy */
    ///     mode: CopyMode  /* Flags controlling behavior of copy */
    /// );
    /// ```
    /// 
    /// The following value may be bitwise ORed in mode to change the behavior of the `copy()` operation:
    /// 
    /// * `UFFDIO_COPY_MODE_DONTWAKE`
    ///        Do not wake up the thread that waits for page-fault resolution
    /// 
    /// Possible errors include:
    /// 
    /// * `EINVAL` Either dst or len was not a multiple of the system page size, or the range specified by src and
    ///        len or dst and len was invalid.
    /// 
    /// * `EINVAL` An invalid bit was specified in the mode field.
    /// 
    /// * `ENOENT` `(since Linux 4.11)`
    ///        The  faulting  process has changed its virtual memory layout simultaneously with an outstanding
    ///        UFFDIO_COPY operation.
    /// 
    /// * `ENOSPC` `(since Linux 4.11)`
    ///        The faulting process has exited at the time of a UFFDIO_COPY operation.
    pub fn copy(&self, dst: *mut u8, src: *mut u8, len: u64, mode: CopyMode) -> Result<(), Error> {
        raw_interface::uffdio_copy(
            self.0,
            raw_interface::defines::uffdio_copy {
                dst: dst as u64,
                src: src as u64,
                len: len,
                mode: mode.bits(),
                copy: 0
            }
        )
    }

    /// `(Since Linux 4.3.)` Zero out a memory range registered with userfaultfd.
    /// 
    /// The requested range is specified by the range field of the `Range` structure
    /// 
    /// The following value may be bitwise ORed in mode to change the behavior of the `UFFDIO_ZERO` operation:
    /// 
    /// * `ZEROPAGE_DONTWAKE` Do not wake up the thread that waits for page-fault resolution.
    /// 
    /// Possible errors include:
    /// 
    /// * `EAGAIN` The  number of bytes zeroed (i.e., the value returned in the zeropage field) does not equal the
    ///        value that was specified in the `range.len` field.
    /// 
    /// * `EINVAL` Either `range.start` or `range.len` was not a multiple of the system page size; or `range.len` was
    ///        zero; or the `range` specified was invalid.
    /// 
    /// * `EINVAL` An invalid bit was specified in the mode field.
    pub fn zeropage<T: Into<Range>>(&self, range: T, mode: ZeropageMode) -> Result<(), Error> {
        raw_interface::uffdio_zeropage(
            self.0,
            raw_interface::defines::uffdio_zeropage {
                range: range.into().into(),
                mode: mode.bits(),
                zeropage: 0
            }
        )
    }
    /// `(Since Linux 4.3.)`  Wake up the thread waiting for page-fault resolution on a specified memory address
    /// range.
    ///
    /// The `wake()` operation is used in conjunction with `copy()` and `zeropage()` operations that
    /// have the `COPY_DONTWAKE` or `ZEROPAGE_DONTWAKE` bit set in the mode field. The
    /// userfault monitor can perform several `copy()` and `zeropage()` operations in a batch and then
    /// explicitly wake up the faulting thread using `wake()`.
    ///
    /// The range argument is a `Range` structure that specifies the address
    /// range.
    ///
    /// Possible errors include:
    ///
    /// * `EINVAL` The  start  or the len field of the `range` structure was not a multiple of the system page
    ///        size; or len was zero; or the specified range was otherwise invalid.
    pub fn wake<T: Into<Range>>(&self, range: T) -> Result<(), Error> {
        raw_interface::uffdio_wake(
            self.0,
            range.into().into()
        )
    }
    #[cfg(feature = "mio")]
    pub fn get_eventfd<'a>(&'a self) -> EventedFd<'a> {
        EventedFd(&self.0)
    }

    pub fn read_message(&self) -> Result<Message, Error> {
        use std::{mem,slice};
        use std::io::Read;
        //let m: uffd_msg = unsafe { std::mem::uninitialized() };
        let mut m: uffd_msg = uffd_msg::default();
        let mut s = self;
        unsafe {
            std::io::Read::read(&mut s, slice::from_raw_parts_mut(&mut m as *mut _ as *mut u8, mem::size_of::<uffd_msg>()))?;
            match m.event as u32 { // TODO: fix the types here
                raw_interface::defines::UFFD_EVENT_PAGEFAULT => {
                    Ok(Message::Pagefault(mem::transmute(m)))
                }
                raw_interface::defines::UFFD_EVENT_FORK => {

                    Ok(Message::Fork(mem::transmute(m)))
                }
                raw_interface::defines::UFFD_EVENT_REMAP => {

                    Ok(Message::Remap(mem::transmute(m)))
                }
                raw_interface::defines::UFFD_EVENT_REMOVE => {

                    Ok(Message::Remove(mem::transmute(m)))
                }
                raw_interface::defines::UFFD_EVENT_UNMAP => {
                    Ok(Message::Unmap(mem::transmute(m)))
                }
                _ => { unimplemented!() }
            }
        }
    }
}

use raw_interface::defines::uffd_msg;

#[derive(Debug)]
pub enum Message {
    Pagefault(PagefaultMessage),
    Fork(ForkMessage),
    Remap(RemapMessage),
    Remove(RemoveMessage),
    Unmap(UnmapMessage)
}

#[derive(Debug)]
#[repr(C)]
pub struct PagefaultMessage {
    _event: u8,
    _res1: u8,
    _res2: u16,
    _res3: u32,
    pub flags: u64,
    pub address: u64,
    _pad: u64
}
#[derive(Debug)]
#[repr(C)]
pub struct ForkMessage {
    _event: u8,
    _res1: u8,
    _res2: u16,
    _res3: u32,
    pub ufd: u32,
    _pad1: u32,
    _pad2: u64,
    _pad3: u64,
}
#[derive(Debug)]
#[repr(C)]
pub struct RemapMessage {
    _event: u8,
    _res1: u8,
    _res2: u16,
    _res3: u32,
    pub from: u64,
    pub to: u64,
    pub len: u64
}
#[derive(Debug)]
#[repr(C)]
pub struct RemoveMessage {
    _event: u8,
    _res1: u8,
    _res2: u16,
    _res3: u32,
    pub start: u64,
    pub end: u64,
    _pad1: u64
}
#[derive(Debug)]
#[repr(C)]
pub struct UnmapMessage {
    _event: u8,
    _res1: u8,
    _res2: u16,
    _res3: u32,
    pub start: u64,
    pub end: u64,
    _pad1: u64
}

#[cfg(test)]
#[test]
fn assert_sizes() {
    use std::mem::size_of;
    assert!(size_of::<PagefaultMessage>() == size_of::<raw_interface::defines::uffd_msg>());
    assert!(size_of::<ForkMessage>() == size_of::<raw_interface::defines::uffd_msg>());
    assert!(size_of::<RemapMessage>() == size_of::<raw_interface::defines::uffd_msg>());
    assert!(size_of::<RemoveMessage>() == size_of::<raw_interface::defines::uffd_msg>());
    assert!(size_of::<UnmapMessage>() == size_of::<raw_interface::defines::uffd_msg>());
}

impl io::Read for Handle {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        raw_interface::read(self.0, buf) 
    }
}

impl<'a> io::Read for &'a Handle {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
            println!("DANGER ZONE");
        raw_interface::read(self.0, buf) 
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        use sc::platform::nr::CLOSE;
        unsafe { syscall!(CLOSE, self.0); }
    }
}

impl AsRawFd for Handle {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl FromRawFd for Handle {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Handle(fd)
    }
}

impl IntoRawFd for Handle {
    fn into_raw_fd(self) -> RawFd {
        let fd = self.as_raw_fd();
        std::mem::forget(self);
        fd
    }
}
