extern crate simplest_session;

use std::{cell::RefCell, rc::Rc, sync::mpsc::{sync_channel, Receiver, SyncSender}, thread::spawn};

use simplest_session::{make_request, HandlerProxy, PackagedRequest, Request, RequestHandler};

struct R1 {
    val: u64,
}

impl Request for R1 {
    type Reply = f64;
}

struct R2 {
    msg: String,
}

impl Request for R2 {
    type Reply = usize;
}

struct Store;

struct Engine {
   _s: Rc<RefCell<Store>>,
}

struct Handle {
    ch: SyncSender<PackagedRequest<Engine>>,
}

impl Handle {
    fn make_req1(&self, val: u64) -> f64 {
        make_request(R1 { val }, &self.ch)
    }

    fn make_req2(&self, msg: String) -> usize {
        make_request(R2 { msg }, &self.ch)
    }
}

impl Engine {
    fn start() -> Handle {
        let (tx, rx): (
            SyncSender<PackagedRequest<Self>>,
            Receiver<PackagedRequest<Self>>,
        ) = sync_channel(16);
        let _ = spawn(move || {
            let mut engine = Engine { _s: Rc::new(RefCell::new(Store)) };
            println!("Starting engine event loop");
            for request in rx.iter() {
                request.run_handler(&mut engine);
            }
            println!("Engine event loop finished.");
        });
        Handle { ch: tx }
    }
}

impl RequestHandler<R1> for Engine {
    fn handle(&mut self, request: &R1) -> <R1 as Request>::Reply {
        request.val as f64 * 0.33
    }
}

impl RequestHandler<R2> for Engine {
    fn handle(&mut self, request: &R2) -> <R2 as Request>::Reply {
        request.msg.len()
    }
}

fn main() {
    let hd = Engine::start();
    let r1 = hd.make_req1(123);
    println!("R1: {}", r1);
    let r2 = hd.make_req2("Hello!".to_owned());
    println!("R2: {}", r2);
}
