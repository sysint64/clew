use std::any::Any;

use tokio::sync::mpsc;

use crate::widgets::builder::BuildContext;

pub struct EventQueue {
    pub(crate) events: Vec<Box<dyn Any + Send>>,
    async_tx: mpsc::UnboundedSender<Box<dyn Any + Send>>,
    async_rx: mpsc::UnboundedReceiver<Box<dyn Any + Send>>,
}

impl EventQueue {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            events: Vec::new(),
            async_tx: tx,
            async_rx: rx,
        }
    }

    pub fn push<E: Any + Send + 'static>(&mut self, event: E) {
        self.events.push(Box::new(event));
    }

    pub fn collect_async_events(&mut self) {
        while let Ok(event) = self.async_rx.try_recv() {
            self.events.push(event);
        }
    }

    pub fn drain(&mut self) -> Vec<Box<dyn Any + Send>> {
        std::mem::take(&mut self.events)
    }
}

impl BuildContext<'_, '_> {
    pub fn emit<E: Any + Send + 'static>(&mut self, event: E) {
        self.event_queue.push(event);
    }

    pub fn spawn<E: Any + Send + 'static, F>(&self, future: F)
    where
        F: Future<Output = E> + Send + 'static,
    {
        let tx = self.event_queue.async_tx.clone();
        tokio::spawn(async move {
            let event = future.await;
            let _ = tx.send(Box::new(event));
        });
    }
}
