use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::{Future, IntoFuture};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use tokio::pin;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::task::JoinHandle;
use crate::caribou::{async_runtime, AsyncTask};
use crate::caribou::gadget::GadgetRef;

#[derive(Debug, Clone, Copy)]
pub enum ValueEventKind { Mutate, Replace, }

pub struct ValueWriteGuard<'a, T: 'static> {
    data: Option<RwLockWriteGuard<'a, T>>,
    value: Value<T>,
}

impl<T: 'static> Deref for ValueWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.data.as_ref().unwrap_unchecked() }
    }
}

impl<T: 'static> DerefMut for ValueWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.data.as_mut().unwrap_unchecked() }
    }
}

impl<T: 'static> Drop for ValueWriteGuard<'_, T> {
    fn drop(&mut self) {
        drop(self.data.take());
        let value = self.value.clone();
        async_runtime().spawn(async move {
            value.notify(ValueEventKind::Mutate).await;
        });
    }
}

type ValueData<T> = Arc<RwLock<T>>;
type ValueListener<T> = Box<dyn Fn(ValueData<T>, ValueEventKind) -> AsyncTask + Send + Sync>;

pub struct Value<T: 'static> {
    data: ValueData<T>,
    listeners: Arc<RwLock<Vec<ValueListener<T>>>>,
}

unsafe impl<T> Send for Value<T> {}
unsafe impl<T> Sync for Value<T> {}

impl<T> Clone for Value<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            listeners: self.listeners.clone(),
        }
    }
}

impl<T> Default for Value<T> where T: Default {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> Debug for Value<T> where T: Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Value")
            .field("data", &self.data)
            .finish()
    }
}

pub fn new_value<T>(data: T) -> Value<T> {
    Value::new(data)
}

impl<T: 'static> Value<T> {
    pub fn new(initial: T) -> Self {
        Self {
            data: Arc::new(RwLock::new(initial)),
            listeners: Default::default(),
        }
    }

    pub async fn get(&self) -> RwLockReadGuard<'_, T> {
        self.data.read().await
    }

    pub async fn get_mut(&self) -> ValueWriteGuard<'_, T> {
        ValueWriteGuard {
            data: Some(self.data.write().await),
            value: self.clone(),
        }
    }

    pub async fn set(&self, value: T) {
        let mut data = self.data.write().await;
        *data = value;
        drop(data);
        self.notify(ValueEventKind::Replace).await;
    }

    pub async fn add_listener(&self, listener: ValueListener<T>) {
        self.listeners.write().await.push(listener);
    }

    pub async fn listen<Func: 'static>(&self, func: Func)
        where Func: Fn(ValueData<T>, ValueEventKind) -> AsyncTask + Send + Sync
    {
        self.add_listener(Box::new(func)).await;
    }

    pub async fn notify(&self, kind: ValueEventKind) {
        for listener in self.listeners.read().await.iter() {
            let task = listener(self.data.clone(), kind);
            task.spawn();
        }
    }
}

impl<T> Value<T> where T: Default {
    pub async fn reset(&self) {
        self.set(Default::default()).await;
    }
}

pub type DynData = Arc<RwLock<Box<dyn Any + Send + Sync>>>;
pub type DynListener = Box<dyn Fn(DynData, ValueEventKind) -> AsyncTask + Send + Sync>;

pub struct DynValue {
    data: DynData,
    listeners: Arc<RwLock<Vec<DynListener>>>,
}

unsafe impl Send for DynValue {}
unsafe impl Sync for DynValue {}

impl Debug for DynValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynValue")
            .field("data", &self.data)
            .finish()
    }
}

struct DynPlaceholder;
static DYN_PLACEHOLDER: DynPlaceholder = DynPlaceholder;

impl Default for DynValue {
    fn default() -> Self {
        Self::new(DynPlaceholder)
    }
}

impl Clone for DynValue {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            listeners: self.listeners.clone(),
        }
    }
}

pub struct DynValueReadGuard<'a, T: Send + Sync> {
    guard: RwLockReadGuard<'a, Box<dyn Any + Send + Sync>>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Send + Sync + 'static> Deref for DynValueReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.guard.downcast_ref::<T>().unwrap() }
    }
}

pub struct DynValueWriteGuard<'a, T: Send + Sync> {
    guard: RwLockWriteGuard<'a, Box<dyn Any + Send + Sync>>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Send + Sync + 'static> Deref for DynValueWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.guard.downcast_ref::<T>().unwrap() }
    }
}

impl<T: Send + Sync + 'static> DerefMut for DynValueWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.guard.downcast_mut::<T>().unwrap() }
    }
}

impl DynValue {
    pub fn new<T: Send + Sync + 'static>(initial: T) -> Self {
        Self {
            data: Arc::new(RwLock::new(Box::new(initial))),
            listeners: Default::default(),
        }
    }

    pub async fn get<T: Send + Sync + 'static>(&self) -> DynValueReadGuard<'_, T> {
        DynValueReadGuard {
            guard: self.data.read().await,
            _phantom: Default::default(),
        }
    }

    pub async fn get_mut<T: Send + Sync + 'static>(&self) -> DynValueWriteGuard<'_, T> {
        DynValueWriteGuard {
            guard: self.data.write().await,
            _phantom: Default::default(),
        }
    }

    pub async fn set<T: Send + Sync + 'static>(&self, value: T) {
        let mut data = self.data.write().await;
        *data = Box::new(value);
        drop(data);
        self.notify(ValueEventKind::Replace).await;
    }

    pub async fn add_listener(&self, listener: DynListener) {
        self.listeners.write().await.push(listener);
    }

    pub async fn listen<Func: 'static>(&self, func: Func)
        where Func: Fn(DynData, ValueEventKind) -> AsyncTask + Send + Sync
    {
        self.add_listener(Box::new(func)).await;
    }

    pub async fn notify(&self, kind: ValueEventKind) {
        for listener in self.listeners.read().await.iter() {
            let task = listener(self.data.clone(), kind);
            task.spawn();
        }
    }

    pub fn exists(&self) -> bool {
        !self.data.blocking_read().is::<DynPlaceholder>()
    }
}

#[derive(Debug, Default)]
pub struct DynValueMap {
    map: Arc<RwLock<HashMap<String, DynValue>>>,
}

impl Clone for DynValueMap {
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
        }
    }
}

impl DynValueMap {
    pub async fn get<T: 'static>(&self, key: &str) -> Option<DynValue> {
        let map = self.map.read().await;
        let value = map.get(key)?;
        Some(value.clone())
    }

    pub async fn set<T: Send + Sync + 'static>(&self, key: &str, value: T) {
        let mut map = self.map.write().await;
        map.insert(key.to_string(), DynValue::new(value));
    }

    pub async fn remove(&self, key: &str) {
        let mut map = self.map.write().await;
        map.remove(key);
    }

    pub async fn contains(&self, key: &str) -> bool {
        let map = self.map.read().await;
        map.contains_key(key)
    }
}