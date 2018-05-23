use std::marker::PhantomData;
use std::sync::mpsc::{sync_channel, SyncSender};

pub trait Request : Send {
    type Reply: Send;
}

pub trait RequestHandler<R>
where
    R: Request,
{
    fn handle(&mut self, request: &R) -> R::Reply;
}

pub trait HandlerProxy: Send {
    type Handler;
    fn run_handler(&self, handler: &mut Self::Handler);
}

pub struct PackagedRequest<H>
{
    inner: Box<HandlerProxy<Handler = H>>,
}

pub fn make_request<R, H>(req: R, ch: &SyncSender<PackagedRequest<H>>) -> R::Reply
where R: Request + 'static,
      H: RequestHandler<R> + 'static,
{
    let (tx, rx) = sync_channel(1);
    let envelope = PackagedRequest {
        inner: Box::new(RequestProxy { req, tx, _hd: PhantomData}),
    };
    let _ = ch.send(envelope);
    let rep = rx.recv().unwrap();
    rep
}

struct RequestProxy<R, H>
where R: Request,
      H: RequestHandler<R>,
{
    req: R,
    tx: SyncSender<R::Reply>,
    _hd: PhantomData<fn(&mut H) -> R::Reply>,
}

impl<R, H> HandlerProxy for RequestProxy<R, H>
where
    R: Request,
    H: RequestHandler<R>,
{
    type Handler = H;
    fn run_handler(&self, hd: &mut Self::Handler) {
        let reply = hd.handle(&self.req);
        let _ = self.tx.send(reply);
    }
}

impl<H> HandlerProxy for PackagedRequest<H>
{
    type Handler = H;
    fn run_handler(&self, hd: &mut Self::Handler) {
        self.inner.run_handler(hd);
    }
}
