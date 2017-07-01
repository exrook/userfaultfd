use ::raw_interface;
use Builder;
use Handle;
use std::os::unix::io::{AsRawFd,FromRawFd,IntoRawFd,RawFd};

//#[test]
//fn it_works() {
//    let fd = UFFDBuilder::new().create().unwrap();
//    println!("fd: {:?}", fd);
//    let mut t = raw_interface::defines::uffdio_api {
//        api: raw_interface::defines::UFFD_API,
//        features: 0,
//        ioctls: 0
//    };
//    let res = raw_interface::uffdio_api(fd.as_raw_fd(), &mut t);
//    println!("res: {:?} t: {:?}", res, t);
//    let mut t2 = raw_interface::defines::uffdio_api {
//        api: raw_interface::defines::UFFD_API,
//        features: t.features,
//        ioctls: 0
//    };
//    println!("t2: {:?}", t2);
//    let res = raw_interface::uffdio_api(fd.as_raw_fd(), &mut t2);
//    println!("res: {:?} t2: {:?}", res, t2);
//}
