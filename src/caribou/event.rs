use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::caribou::{AsyncTask};

#[deprecated]
pub struct Event<F: ?Sized + Send + Sync> {
    handlers: Arc<RwLock<Vec<Box<F>>>>,
}

impl<F: ?Sized + Send + Sync> Default for Event<F> {
    fn default() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl<F: ?Sized + Send + Sync> Debug for Event<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Event").finish()
    }
}

impl<F: ?Sized + Send + Sync> Event<F> {
    pub async fn add_handler(&self, handler: Box<F>) {
        self.handlers.write().await.push(handler);
    }

    pub async fn remove_at(&self, index: usize) {
        self.handlers.write().await.remove(index);
    }
}

pub type ZeroArgEvent<R = ()> = Event<dyn Fn() -> AsyncTask<R> + Send + Sync>;

impl<R: 'static + Send + Sync> ZeroArgEvent<R> {
    pub async fn gather(&self) -> Vec<R> {
        let handlers = self.handlers.read().await;
        let futures = handlers.iter()
            .map(|handler| (handler)())
            .map(|task| task.spawn());
        let mut results = Vec::new();
        for future in futures {
            results.push(future.await.unwrap());
        }
        results
    }

    pub async fn handle<F>(&self, handler: F)
        where F: Fn() -> AsyncTask<R> + Send + Sync + 'static
    {
        self.add_handler(Box::new(handler)).await;
    }
}

impl ZeroArgEvent {
    pub async fn broadcast(&self) {
        let handlers = self.handlers.read().await;
        let futures = handlers.iter()
            .map(|handler| (handler)())
            .map(|task| task.spawn());
        for future in futures {
            future.await.unwrap();
        }
    }
}

pub type SingleArgEvent<P, R = ()> = Event<dyn Fn(P) -> AsyncTask<R> + Send + Sync>;

impl<P: Clone + 'static + Send + Sync, R: 'static + Send + Sync> SingleArgEvent<P, R> {
    pub async fn gather(&self, param: P) -> Vec<R> {
        let handlers = self.handlers.read().await;
        let futures = handlers.iter()
            .map(|handler| (handler)(param.clone()))
            .map(|task| task.spawn());
        let mut results = Vec::new();
        for future in futures {
            results.push(future.await.unwrap());
        }
        results
    }

    pub async fn handle<F>(&self, handler: F)
        where F: Fn(P) -> AsyncTask<R> + Send + Sync + 'static
    {
        self.add_handler(Box::new(handler)).await;
    }
}

impl<P: Clone + 'static + Send + Sync> SingleArgEvent<P> {
    pub async fn broadcast(&self, param: P) {
        let handlers = self.handlers.read().await;
        let futures = handlers.iter()
            .map(|handler| (handler)(param.clone()))
            .map(|task| task.spawn());
        for future in futures {
            future.await.unwrap();
        }
    }
}