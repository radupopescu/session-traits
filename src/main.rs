extern crate session_traits;

use std::{cell::RefCell,
          rc::Rc,
          sync::mpsc::{channel, Receiver, Sender},
          thread::{spawn, JoinHandle}};

use session_traits::{make_request, HandlerProxy, PackagedRequest, Request, RequestHandler};

struct R1 {
    val: i64,
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
    ch: Sender<PackagedRequest<Engine>>,
}

impl Handle {
    fn make_req1(&self, val: i64) -> f64 {
        make_request(R1 { val }, &self.ch)
    }

    fn make_req2(&self, msg: String) -> usize {
        make_request(R2 { msg }, &self.ch)
    }
}

impl Engine {
    fn start() -> (Handle, JoinHandle<()>) {
        let (tx, rx): (
            Sender<PackagedRequest<Self>>,
            Receiver<PackagedRequest<Self>>,
        ) = channel();
        let thread_handle = spawn(move || {
            let mut engine = Engine {
                _s: Rc::new(RefCell::new(Store)),
            };
            println!("Starting engine event loop");
            for request in rx.iter() {
                request.run_handler(&mut engine);
            }
            println!("Engine event loop finished.");
        });
        (Handle { ch: tx }, thread_handle)
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
    let th = {
        let (hd, th) = Engine::start();
        let r1 = hd.make_req1(123);
        println!("R1: {}", r1);
        let r2 = hd.make_req2("Hello!".to_owned());
        println!("R2: {}", r2);
        th
    };
    let _ = th.join();
}
