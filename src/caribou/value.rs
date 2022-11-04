use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::mem;
use crate::as_clone;
use crate::caribou::event::{Event, Listener, PinnedFutureBox};

pub struct Value<T: Send + Clone> {
    value: Arc<Mutex<T>>,
    event: Event<(T, Value<T>), ()>,
}

impl<T: Send + Clone> Clone for Value<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            event: self.event.clone(),
        }
    }
}

impl<T: Send + Clone + Debug> Debug for Value<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Value")
            .field("value", &self.value)
            .field("event", &self.event)
            .finish()
    }
}

impl<T: Send + Clone> Value<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Arc::new(Mutex::new(value)),
            event: Event::new(),
        }
    }

    pub async fn get(&self) -> T {
        self.value.lock().await.clone()
    }

    pub async fn set(&self, value: T) {
        let mut lock = self.value.lock().await;
        let old = mem::replace(&mut *lock, value);
        drop(lock);
        self.event.emit((old, self.clone())).await;
    }

    pub async fn listen<F>(&self, callback: F) -> Listener<(T, Value<T>), ()>
        where F: Fn((T, Value<T>)) -> PinnedFutureBox<()> + Send + Sync + 'static
    {
        self.event.listen(callback).await
    }
}

pub struct DerivedValue<T: Send + Clone> {
    value: Value<Option<T>>,
    producer: Arc<dyn Fn() -> PinnedFutureBox<T> + Send + Sync>,
}

impl<T: Send + Clone> Clone for DerivedValue<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            producer: self.producer.clone(),
        }
    }
}

impl<T: Send + Clone> DerivedValue<T> {
    pub async fn new(producer: impl Fn() -> PinnedFutureBox<T> + Send + Sync + 'static) -> Self {
        Self {
            value: Value::new(None),
            producer: Arc::new(producer),
        }
    }

    pub fn value(&self) -> &Value<Option<T>> {
        &self.value
    }

    pub async fn get(&self) -> T {
        if let Some(value) = self.value.get().await {
            value
        } else {
            let value = (self.producer)().await;
            self.value.set(Some(value.clone())).await;
            value
        }
    }

    pub async fn update(&self) {
        let value = (self.producer)().await;
        self.value.set(Some(value)).await;
    }

    pub async fn hook<U: Send + Clone + 'static>(&self, other: &Value<U>) where T: 'static {
        let value = self.clone();
        other.listen(move |_| {
            as_clone!(value);
            Box::pin(async move {
                value.update().await;
            })
        }).await;
    }
}
