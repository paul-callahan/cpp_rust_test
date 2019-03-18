use std::sync::mpsc::{sync_channel, SyncSender, Receiver, channel};
use std::{thread, panic, ptr, mem};
use std::time::{SystemTime, UNIX_EPOCH, Instant};

use crossbeam_channel::{TrySendError, unbounded};
use crossbeam_channel::TryRecvError;
use crossbeam_channel::Sender;

//use std::sync::mpsc::TryRecvError;
//use std::sync::mpsc::TrySendError;
//use std::sync::mpsc::Sender;

use std::thread::JoinHandle;
use std::sync::atomic::{AtomicUsize, Ordering};

#[macro_use]
extern crate crossbeam_channel;
use crossbeam_channel::bounded;

const BATCH: u128 = 1000000;

#[no_mangle]
pub extern "C" fn create_sender_receiver() -> *mut SenderReceiver {
    ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
        let sr = SenderReceiver::new();
        sr.send_msg("1. test message from inside of rust".to_string());


        Box::into_raw(Box::new(sr))
    })).unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn send_msg(sr: *mut SenderReceiver) {
    let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
        send_msg_panic(sr);
    }));
    match result {
        Err(e) => {
            eprintln!("send_msg got error: {:?}", e)
        }
        _ => ()
    }
}

fn send_msg_panic(sr: *mut SenderReceiver) {
    let mut sr: Box<SenderReceiver> = unsafe { Box::from_raw(sr) };
    sr.send_msg("A msg from CPP !!!".to_string());
    mem::forget(sr);
}


#[repr(C)]
pub struct SenderReceiver {
    //sender: SyncSender<String>,
    sender: Sender<String>,
    _receiver_thread: JoinHandle<()>,
    drop_count: AtomicUsize,
    success_count: AtomicUsize,

}

impl SenderReceiver {
    fn new() -> SenderReceiver {
        //let (sender, receiver) = sync_channel(1000);
        //let (sender, receiver) = channel(); //sync_channel(1000000);
        let (sender, receiver) =  bounded(1000);
	//let (sender, receiver) =  unbounded();

        let _receiver_thread: JoinHandle<()> = thread::spawn(move || {
            //let thread_panic = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
            let mut i = 0;
            let mut now = Instant::now();
            let thread_panic = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                loop {
                    let res = receiver.try_recv();
                    match res {
                        Ok(msg) => {
                            i += 1;
                            if i % BATCH == 0 {
                                let new_now = Instant::now();
                                let duration = new_now.duration_since(now).as_millis();
                                now = new_now;
                                println!("RECV tp: {} msgs/ms", (BATCH / duration));
                                println!("receiving: <{}> - {}", msg, i);

                            }
                        }
                        Err(err) if err == TryRecvError::Empty => {
                            //eprintln!("RECV got error: {:?} {}", err, i);
                        }
                        Err(err) => {
                            eprintln!("RECV got error: {:?} {}", err, i);
                        }
                    }
                    //std::thread::sleep_ms(1);
                }
            }));
            match thread_panic {
                Err(e) => { eprintln!("thread panic: {:?}", e); }
                _ => {}
            };
        });

        SenderReceiver {
            sender,
            _receiver_thread,
            drop_count: AtomicUsize::new(0),
            success_count: AtomicUsize::new(0),
        }
    }


    fn send_msg(&self, msg: String) {
       // self.async_send(msg);

        self.sync_send(msg);

        ///////////

    }

    fn async_send(&self, msg: String) {
        let result = self.sender.send(msg);
        match result {
            Err(err) => { eprintln!("SEND got error: <{:?}>", err); }
            _ => {}
        }
    }

    fn sync_send(&self, msg: String) {
        let result = self.sender.try_send(msg);
        match result {
            Err(err) => {
                match err {
                    TrySendError::Disconnected(emsg) => {
                        eprintln!("SEND got error: <{:?}>", emsg);
                    }
                    _ => {
                        /*full*/
                        let drops = self.drop_count.fetch_add(1, Ordering::SeqCst);
                        if drops % 1000000 == 0 {
                            let successes = self.success_count.load(Ordering::Relaxed);
                            println!("drop rate:  {} drops / successes", (drops/successes));
                        }
                    }
                }
            }

            _ => {
                self.success_count.fetch_add(1, Ordering::SeqCst);
                //println!("---------------sender.try_send() returned ok");
            }
        }
    }
}