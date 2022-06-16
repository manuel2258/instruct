use crossbeam_channel::{unbounded, Receiver, Sender};

pub struct TwoWayChannel<ReqT, RespT> {
    pub tx: Sender<ReqT>,
    pub rx: Receiver<RespT>,
}

impl<ReqT, RespT> TwoWayChannel<ReqT, RespT> {
    pub fn new_pair() -> (TwoWayChannel<ReqT, RespT>, TwoWayChannel<RespT, ReqT>) {
        let (s1, r1) = unbounded();
        let (s2, r2) = unbounded();
        (
            TwoWayChannel { tx: s1, rx: r2 },
            TwoWayChannel { tx: s2, rx: r1 },
        )
    }
}
