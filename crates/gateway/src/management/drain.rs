use tokio::sync::watch;

#[derive(Clone)]
pub struct DrainSignal(watch::Sender<bool>);

#[derive(Clone)]
pub struct DrainWatcher(watch::Receiver<bool>);

impl DrainSignal {
    pub fn new() -> (Self, DrainWatcher) {
        let (tx, rx) = watch::channel(false);
        (Self(tx), DrainWatcher(rx))
    }

    pub fn signal(&self) {
        self.0.send_replace(true);
    }
}

impl DrainWatcher {
    pub(crate) async fn signaled(mut self) {
        if *self.0.borrow_and_update() {
            return;
        }
        while self.0.changed().await.is_ok() {
            if *self.0.borrow_and_update() {
                return;
            }
        }
    }
}
