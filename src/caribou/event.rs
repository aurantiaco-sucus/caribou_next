use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::pin::Pin;
use std::future::Future;
use crate::caribou::input::MouseButton;
use crate::caribou::math::ScalarPair;

pub type PinnedFutureBox<R> = Pin<Box<dyn Future<Output=R>>>;
pub type Listener<P, R> = Arc<dyn Fn(P) -> PinnedFutureBox<R>  + Send + Sync>;

pub struct Event<P: Send + Clone = (), R: Send + Clone = ()> {
    funcs: Arc<Mutex<Vec<Listener<P, R>>>>
}

impl<P: Send + Clone, R: Send + Clone> Clone for Event<P, R> {
    fn clone(&self) -> Self {
        Self {
            funcs: self.funcs.clone(),
        }
    }
}

impl<P: Send + Clone, R: Send + Clone> Debug for Event<P, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Event")
            .finish()
    }
}

impl<P: Send + Clone, R: Send + Clone> Event<P, R> {
    pub fn new() -> Self {
        Self {
            funcs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn listen(&self, callback: impl Fn(P) -> PinnedFutureBox<R> + Send + Sync + 'static) -> Listener<P, R> {
        let callback = Arc::new(callback);
        self.funcs.lock().await.push(callback.clone());
        callback
    }

    pub async fn emit(&self, param: P) -> Vec<R> {
        let listeners = self.funcs.lock().await.clone();
        let futures = listeners
            .into_iter()
            .map(|f| f(param.clone()));
        let mut results = Vec::new();
        for func in futures {
            results.push(func.await);
        }
        results
    }
}