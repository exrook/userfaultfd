extern crate userfaultfd;
extern crate memmap;
extern crate mio;

use std::thread;
use std::time::Duration;

use mio::{Poll, PollOpt, Token, Ready, Events};
use userfaultfd::{Builder,Message,CopyMode,Range,REGISTER_MISSING};
use memmap::{Mmap,Protection};

fn main() {
    let (mut fd, ioctls) = Builder::new().non_block(true).close_on_exec(true).create().expect("LUL");
    println!("{:?}", fd);
    println!("{}", ioctls);
    let mut map = Mmap::anonymous(500*4096, Protection::ReadWrite).unwrap();
    println!("{:?}", map);
    let o = fd.register(unsafe { map.as_mut_slice() }, REGISTER_MISSING).expect("Register failed");
    println!("register: {:?}", o);
    let t = thread::spawn(||lul(map));
    let eventfd = fd.get_eventfd();
    println!("{:?}", eventfd);
    let poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1);
    poll.register(&eventfd, Token(1), Ready::readable(), PollOpt::edge()).unwrap();

    let mut zeropage = Mmap::anonymous(4096, Protection::ReadWrite).unwrap();

    loop {
        let res = poll.poll(&mut events, Some(Duration::from_millis(100))).expect("Error in poll()");
        for e in events.iter() {
            let (readiness, token) = (e.readiness(), e.token());
            match token {
                Token(1) => {
                    use Message::*;
                    match fd.read_message() {
                        Ok(Pagefault(p)) => {
                            println!("{:?}", p);
                            let range = Range {
                                start: p.address as *mut u8,
                                len: 4096
                            };
                            println!("{:x}", p.address);
                            fd.copy(p.address as *mut u8, zeropage.mut_ptr(), 4096, CopyMode::empty());
                            //fd.zeropage(range,ZeropageMode::empty()).expect("Zeropage call failed");
                            println!("Filled request");
                        }
                        Ok(Fork(p)) => {
                            println!("{:?}", p);
                        }
                        Ok(Remap(p)) => {
                            println!("{:?}", p);
                        }
                        Ok(Remove(p)) => {
                            println!("{:?}", p);
                        }
                        Ok(Unmap(p)) => {
                            println!("{:?}", p);
                        }
                        Err(e) => {
                            println!("{:?}", e)
                        }
                    }
                }
                _ => {}
            }
        }
    }
    //t.join().unwrap();
}

fn lul(mut map: Mmap) {
    thread::sleep(Duration::new(2,0));
    println!("thread2");
    for i in 0..500 {
        println!("Value: {:x}", unsafe { map.as_mut_slice()[i*4096 + (i%4096)]  });
    }
}
