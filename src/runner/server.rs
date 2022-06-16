use std::thread::{self, JoinHandle};

use crate::util::channel::TwoWayChannel;

use super::{
    message::{RunnerRequest, RunnerResponse},
    router::Router,
};

pub type RunnerChannel = TwoWayChannel<RunnerResponse, RunnerRequest>;

pub struct RunnerServer {
    channel: RunnerChannel,
    router: Router,
}

impl RunnerServer {
    pub fn new(channel: RunnerChannel) -> Self {
        Self {
            channel,
            router: Router::new(),
        }
    }

    pub fn new_thread(channel: RunnerChannel) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut server = RunnerServer::new(channel);
            server.serve();
        })
    }

    pub fn serve(&mut self) {
        while let Ok(request) = self.channel.rx.recv() {
            let response = self.router.handle_request(request);

            if let Err(_) = self.channel.tx.send(response) {
                return;
            }
        }
    }
}
