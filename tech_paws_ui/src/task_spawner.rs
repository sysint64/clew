use tokio::sync::mpsc;

pub struct TaskSpawner {
    redraw_tx: mpsc::UnboundedSender<()>,
}

impl TaskSpawner {
    pub fn new(redraw_tx: mpsc::UnboundedSender<()>) -> Self {
        Self { redraw_tx }
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let redraw_tx = self.redraw_tx.clone();
        tokio::spawn(async move {
            future.await;
            let _ = redraw_tx.send(());
        });
    }

    pub fn spawn_with_result<F, R>(&self, future: F) -> tokio::task::JoinHandle<R>
    where
        F: std::future::Future<Output = R> + Send + 'static,
        R: Send + 'static,
    {
        let redraw_tx = self.redraw_tx.clone();
        tokio::spawn(async move {
            let result = future.await;
            let _ = redraw_tx.send(());
            result
        })
    }
}
