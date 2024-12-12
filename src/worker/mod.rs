use std::{
    ops::Deref,
    sync::{Arc, Mutex, RwLock},
};

use futures::{future::OptionFuture, task::ArcWake, StreamExt};
use lapin::{options::BasicConsumeOptions, types::FieldTable, ConnectionProperties};
use tokio::runtime::Runtime;

use crate::{app::config::RabbitMQSettings, trace_err, types::json_log::LogEntry};

mod cmd_rcv;

#[derive(Default, Clone)]
struct ConnWaker(Arc<Mutex<Option<std::task::Waker>>>);

impl ConnWaker {
    fn put(&self, w: std::task::Waker) {
        let mut guard = self.0.lock().expect("Failed to lock ConnWaker guard");
        guard.replace(w);
    }

    /// Wake will deinit waker (take it off), and then we can reuse it again
    fn wake(&self) {
        if let Ok(mut guard) = self.0.lock() {
            if let Some(waker) = guard.take() {
                waker.wake();
            }
        }
    }
}

pub enum Command {
    Reconnect,
    UpdateConfig(RabbitMQSettings),
}

#[derive(Debug)]
pub enum Notification {
    LogEntry(LogEntry),
    ConnectionStatusChanged { status: Result<(), String> },
    Error(String),
}

pub struct WorkerHandle {
    tx: crossbeam::channel::Sender<Command>,
    rx: std::sync::mpsc::Receiver<Notification>,
    waker: Arc<RwLock<Option<std::task::Waker>>>,
}

impl WorkerHandle {
    pub fn command(&self, cmd: Command) {
        self.tx.send(cmd).expect("Failed to send command!");
        let lock = self
            .waker
            .write()
            .expect("Failed to acquire lock waker for write")
            .take()
            .expect("Waker was None");
        lock.wake();
    }
    pub fn get_notifications(&self) -> Vec<Notification> {
        let mut notifications = Vec::new();
        loop {
            match self.rx.try_recv() {
                Ok(n) => notifications.push(n),
                Err(err) => match err {
                    std::sync::mpsc::TryRecvError::Empty => return notifications,
                    std::sync::mpsc::TryRecvError::Disconnected => {
                        panic!("Disconnected from worker!")
                    }
                },
            }
        }
    }
}

pub struct Worker {
    ctx: egui::Context,
    rmq_conf: RabbitMQSettings,
    ntf_tx: std::sync::mpsc::Sender<Notification>,
    cmd_rx: crossbeam::channel::Receiver<Command>,
    worker_handle: Option<WorkerHandle>,
    worker_handle_waker: Arc<RwLock<Option<std::task::Waker>>>,
    conn_waker: ConnWaker,
}

impl Worker {
    pub fn new(config: RabbitMQSettings, ctx: egui::Context) -> Self {
        let (cmd_tx, cmd_rx) = crossbeam::channel::unbounded();
        let (ntf_tx, ntf_rx) = std::sync::mpsc::channel();

        let waker = Arc::new(RwLock::new(None));
        Worker {
            ctx,
            rmq_conf: config,
            ntf_tx,
            cmd_rx,
            worker_handle: Some(WorkerHandle {
                tx: cmd_tx,
                rx: ntf_rx,
                waker: waker.clone(),
            }),
            conn_waker: Default::default(),
            worker_handle_waker: waker,
        }
    }

    pub fn start(mut self) -> WorkerHandle {
        let wh = self.worker_handle.take().unwrap();
        let rt = Runtime::new().expect("Unable to create Runtime");

        // Execute the runtime in its own thread.
        std::thread::spawn(move || {
            rt.block_on(async {
                let mut cons = self.connect().await;
                loop {
                    tokio::select! {
                        // biased;
                        Some(cmd) = self.get_cmd() => {
                            match cmd {
                                Command::Reconnect => {
                                    cons = self.connect().await;
                                },
                                Command::UpdateConfig(c) => {
                                    self.rmq_conf = c;
                                },
                            }
                        }
                        Some((Some(Ok(delivery)), c)) = OptionFuture::from(cons.take().map(|c| c.into_future())) => {
                            cons = Some(c);
                            // FIXME: Sometimes message not acknowledged.
                            // Why?
                            trace_err!(delivery.ack(lapin::options::BasicAckOptions::default())
                            .await, ());
                            let entry = match serde_json::from_slice(&delivery.data) {
                                Ok(entry) => entry,
                                Err(err) => {
                                    self.ntf_tx
                                        .send(Notification::Error(err.to_string()))
                                        .expect("Failed to send msg!");
                                    continue;
                                }
                            };
                            self.notify(Notification::LogEntry(entry));
                        }
                        else => {
                            println!("Exiting from worker!");
                            break;
                        }
                    }
                }
            })
        });
        wh
    }

    fn notify(&self, n: Notification) {
        self.ntf_tx.send(n).expect("Failed to send notification");
        self.ctx.request_repaint();
    }

    async fn connect(&self) -> Option<lapin::Consumer> {
        let notify_err = |e: Box<dyn std::error::Error>| {
            self.notify(Notification::ConnectionStatusChanged {
                status: Err(e.to_string()),
            });
        };
        let conn = lapin::Connection::connect(
            &self.rmq_conf.connection_string(),
            ConnectionProperties::default(),
        )
        .await;
        let conn = match conn {
            Ok(c) => c,
            Err(e) => {
                notify_err(Box::new(e));
                return None;
            }
        };
        let ch = match conn.create_channel().await {
            Ok(ch) => ch,
            Err(e) => {
                notify_err(Box::new(e));
                return None;
            }
        };
        let cons = match ch
            .basic_consume(
                "log",
                "",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
        {
            Ok(cons) => cons,
            Err(e) => {
                notify_err(Box::new(e));
                return None;
            }
        };
        self.conn_waker.wake();
        self.notify(Notification::ConnectionStatusChanged { status: Ok(()) });
        Some(cons)
    }

    fn get_cmd(&self) -> cmd_rcv::CmdRcv {
        cmd_rcv::CmdRcv {
            waker: self.worker_handle_waker.clone(),
            rx: self.cmd_rx.clone(),
        }
    }
}
