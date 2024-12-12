use std::future::Future;
use std::sync::{Arc, RwLock};
use std::task::Poll;

use crossbeam::channel::TryRecvError;

pub struct CmdRcv {
    pub waker: Arc<RwLock<Option<std::task::Waker>>>,
    pub rx: crossbeam::channel::Receiver<super::Command>,
}

// FIXME: Not resolves command in time
// Review that clearly
impl Future for CmdRcv {
    type Output = Option<super::Command>;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.rx.try_recv() {
            Ok(cmd) => Poll::Ready(Some(cmd)),
            Err(TryRecvError::Empty) => {
                let mut lock = self
                    .waker
                    .write()
                    .expect("Failed to acquire lock waker for writing");
                lock.replace(cx.waker().clone());
                Poll::Pending
            }
            Err(TryRecvError::Disconnected) => Poll::Ready(None),
        }
    }
}
