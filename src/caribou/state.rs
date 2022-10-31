use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::hash::Hash;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::{Arc};
use tokio::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use crate::caribou::async_runtime;
use crate::caribou::gadget::GadgetRef;

pub struct Arbitrary {
    data: Arc<Box<dyn Any + Send + Sync>>,
}

pub struct ArbitraryPlaceholder;

impl Clone for Arbitrary {
    fn clone(&self) -> Self {
        Arbitrary { data: self.data.clone() }
    }
}

impl Debug for Arbitrary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Arbitrary").finish()
    }
}

impl PartialEq for Arbitrary {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }
}

impl Arbitrary {
    pub fn new<T: Any + Send + Sync>(data: T) -> Self {
        Self {
            data: Arc::new(Box::new(data)),
        }
    }

    pub fn placeholder() -> Self {
        Self {
            data: Arc::new(Box::new(ArbitraryPlaceholder)),
        }
    }

    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.data.downcast_ref::<T>()
    }

    pub fn ptr_eq<T: Any + Send + Sync>(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }

    pub fn partial_eq<T: Any + Send + Sync + PartialEq>(&self, other: &T) -> bool {
        self.data.downcast_ref::<T>()
            .map(|data| data == other)
            .unwrap_or(false)
    }

    pub fn is<T: Any + Send + Sync>(&self) -> bool {
        self.data.downcast_ref::<T>().is_some()
    }

    pub fn is_placeholder(&self) -> bool {
        self.data.downcast_ref::<ArbitraryPlaceholder>().is_some()
    }
}

pub struct MutableArbitrary {
    data: Arc<Mutex<dyn Any + Send + Sync>>,
}

impl Clone for MutableArbitrary {
    fn clone(&self) -> Self {
        MutableArbitrary { data: self.data.clone() }
    }
}

impl Debug for MutableArbitrary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Arbitrary").finish()
    }
}

impl PartialEq for MutableArbitrary {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }
}

impl MutableArbitrary {
    pub fn new<T: Any + Send + Sync>(data: T) -> Self {
        Self {
            data: Arc::new(Mutex::new(data)),
        }
    }

    pub fn placeholder() -> Self {
        Self {
            data: Arc::new(Mutex::new(ArbitraryPlaceholder)),
        }
    }

    pub async fn get<T: Any + Send + Sync>(&self) -> MutableArbitraryReadGuard<T> {
        MutableArbitraryReadGuard {
            guard: self.data.lock().await,
            _phantom: Default::default(),
        }
    }

    pub async fn get_mut<T: Any + Send + Sync>(&self) -> MutableArbitraryWriteGuard<T> {
        MutableArbitraryWriteGuard {
            guard: self.data.lock().await,
            _phantom: Default::default(),
        }
    }

    pub async fn set<T: Any + Send + Sync>(&self, new_data: T) -> T {
        let mut guard = self.data.lock().await;
        let data = guard.downcast_mut().unwrap();
        mem::replace(data, new_data)
    }

    pub async fn ptr_eq<T: Any + Send + Sync>(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }

    pub async fn partial_eq<T: Any + Send + Sync + PartialEq>(&self, other: &T) -> bool {
        let guard = self.data.lock().await;
        guard.downcast_ref::<T>()
            .map(|data| data == other)
            .unwrap_or(false)
    }

    pub async fn is<T: Any + Send + Sync>(&self) -> bool {
        let guard = self.data.lock().await;
        guard.downcast_ref::<T>().is_some()
    }

    pub async fn is_placeholder(&self) -> bool {
        let guard = self.data.lock().await;
        guard.downcast_ref::<ArbitraryPlaceholder>().is_some()
    }
}

pub struct MutableArbitraryReadGuard<'a, T: Any + Send + Sync> {
    guard: MutexGuard<'a, dyn Any + Send + Sync>,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T: Any + Send + Sync> Deref for MutableArbitraryReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.downcast_ref::<T>().unwrap()
    }
}

pub struct MutableArbitraryWriteGuard<'a, T: Any + Send + Sync> {
    guard: MutexGuard<'a, dyn Any + Send + Sync>,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T: Any + Send + Sync> Deref for MutableArbitraryWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.downcast_ref::<T>().unwrap()
    }
}

impl<'a, T: Any + Send + Sync> DerefMut for MutableArbitraryWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.downcast_mut::<T>().unwrap()
    }
}

pub type Listener<E> = Box<dyn Fn(E) -> Pin<Box<dyn Future<Output=()> + Send + Sync>> + Send + Sync>;
pub type Listeners<E> = Arc<RwLock<Vec<Listener<E>>>>;
pub type ListenerIdentities = Arc<RwLock<Vec<&'static str>>>;

pub struct State<T: Send + Sync> {
    data: Arc<RwLock<T>>,
    gadget: GadgetRef,
    listeners: Listeners<StateChangedEvent<T>>,
    identities: ListenerIdentities,
}

impl<T: Send + Sync> Clone for State<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            gadget: self.gadget.clone(),
            listeners: self.listeners.clone(),
            identities: self.identities.clone(),
        }
    }
}

pub struct StateChangedEvent<T: Send + Sync> {
    pub state: State<T>,
    pub gadget: GadgetRef,
}

impl<T: Send + Sync> Clone for StateChangedEvent<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            gadget: self.gadget.clone(),
        }
    }
}

impl<T: Send + Sync> State<T> {
    pub fn new(gadget: GadgetRef, data: T) -> Self {
        Self {
            data: Arc::new(RwLock::new(data)),
            gadget,
            listeners: Arc::new(RwLock::new(Vec::new())),
            identities: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn new_from<U: Into<T>>(gadget: GadgetRef, data: U) -> Self {
        Self::new(gadget, data.into())
    }

    pub async fn get(&self) -> RwLockReadGuard<'_, T> {
        self.data.read().await
    }

    pub async fn get_cloned(&self) -> T
    where
        T: Clone,
    {
        self.data.read().await.clone()
    }

    pub async fn get_mut(&self) -> RwLockWriteGuard<'_, T> {
        self.data.write().await
    }

    pub async fn set(&self, data: T) where T: Clone {
        let mut lock = self.data.write().await;
        *lock = data;
        self.notify().await;
    }

    pub async fn set_from<U: Into<T>>(&self, data: U) where T: Clone {
        self.set(data.into()).await;
    }

    pub async fn listen(&self, name: &'static str, listener: impl Fn(StateChangedEvent<T>) ->
        Pin<Box<dyn Future<Output=()> + Send + Sync>> + Send + Sync + 'static) {
        self.listeners.write().await.push(Box::new(listener));
    }

    pub async fn remove_listener(&self, name: &'static str) {
        let mut listeners = self.listeners.write().await;
        let mut identities = self.identities.write().await;
        if let Some(index) = identities.iter().position(|identity| identity == &name) {
            listeners.remove(index);
            identities.remove(index);
        }
    }

    pub async fn notify(&self) {
        let event = StateChangedEvent {
            state: self.clone(),
            gadget: self.gadget.clone(),
        };
        let listeners = self.listeners.read().await;
        for listener in listeners.iter() {
            async_runtime().spawn((listener)(event.clone()));
        }
    }
}

impl State<Arbitrary> {
    pub fn new_any<T: Any + Send + Sync>(gadget: GadgetRef, data: T) -> Self {
        Self::new(gadget, Arbitrary::new(data))
    }

    pub async fn get_as<T: Any + Send + Sync>(&self) -> ArbitraryStateReadGuard<T>
    where
        T: Clone,
    {
        ArbitraryStateReadGuard {
            data: self.get_cloned().await,
            _marker: Default::default()
        }
    }

    pub async fn set_any<T: Any + Send + Sync>(&self, data: T) {
        let mut lock = self.data.write().await;
        *lock = Arbitrary::new(data);
        self.notify().await;
    }
}

pub struct ArbitraryStateReadGuard<T> {
    data: Arbitrary,
    _marker: std::marker::PhantomData<T>,
}

impl<T: 'static + Sync + Send> Deref for ArbitraryStateReadGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data.data.downcast_ref::<T>().unwrap()
    }
}

impl State<MutableArbitrary> {
    pub fn new_any_mut<T: Any + Send + Sync>(gadget: GadgetRef, data: T) -> Self {
        Self::new(gadget, MutableArbitrary::new(data))
    }

    pub async fn set_any<T: Any + Send + Sync>(&self, data: T) {
        let mut lock = self.data.write().await;
        *lock = MutableArbitrary::new(data);
        self.notify().await;
    }
}

pub struct OptionalState<T: Send + Sync + Clone> {
    data: Arc<RwLock<Option<T>>>,
    gadget: GadgetRef,
    on_set: Listeners<OptionalStateSetEvent<T>>,
    on_unset: Listeners<OptionalStateUnsetEvent<T>>,
    on_change: Listeners<OptionalStateChangedEvent<T>>,
    id_set: ListenerIdentities,
    id_unset: ListenerIdentities,
    id_change: ListenerIdentities,
}

impl<T: Send + Sync + Clone> Clone for OptionalState<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            gadget: self.gadget.clone(),
            on_set: self.on_set.clone(),
            on_unset: self.on_unset.clone(),
            on_change: self.on_change.clone(),
            id_set: Arc::new(Default::default()),
            id_unset: Arc::new(Default::default()),
            id_change: Arc::new(Default::default())
        }
    }
}

pub struct OptionalStateSetEvent<T: Send + Sync + Clone> {
    pub state: OptionalState<T>,
    pub gadget: GadgetRef,
    pub value: T,
}

impl<T: Send + Sync + Clone> Clone for OptionalStateSetEvent<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            gadget: self.gadget.clone(),
            value: self.value.clone(),
        }
    }
}

pub struct OptionalStateUnsetEvent<T: Send + Sync + Clone> {
    pub state: OptionalState<T>,
    pub last_value: T,
    pub gadget: GadgetRef,
}

impl<T: Send + Sync + Clone> Clone for OptionalStateUnsetEvent<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            last_value: self.last_value.clone(),
            gadget: self.gadget.clone(),
        }
    }
}

pub struct OptionalStateChangedEvent<T: Send + Sync + Clone> {
    pub state: OptionalState<T>,
    pub last_value: T,
    pub new_value: T,
    pub gadget: GadgetRef,
}

impl<T: Send + Sync + Clone> Clone for OptionalStateChangedEvent<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            last_value: self.last_value.clone(),
            new_value: self.new_value.clone(),
            gadget: self.gadget.clone(),
        }
    }
}

impl<T: Send + Sync + Clone> OptionalState<T> {
    pub fn new(gadget: GadgetRef, data: Option<T>) -> Self {
        Self {
            data: Arc::new(RwLock::new(data)),
            gadget,
            on_set: Listeners::default(),
            on_unset: Listeners::default(),
            on_change: Listeners::default(),
            id_set: Arc::new(Default::default()),
            id_unset: Arc::new(Default::default()),
            id_change: Arc::new(Default::default())
        }
    }

    pub fn new_from<U: Into<Option<T>>>(gadget: GadgetRef, data: U) -> Self {
        Self::new(gadget, data.into())
    }

    pub fn new_empty(gadget: GadgetRef) -> Self {
        Self::new(gadget, None)
    }

    pub async fn get(&self) -> Option<T> {
        self.data.read().await.clone()
    }

    pub async fn put(&self, data: T) {
        let mut lock = self.data.write().await;
        let old_value = lock.take();
        *lock = Some(data.clone());
        if old_value.is_none() {
            self.notify_set(data).await;
        } else {
            self.notify_change(old_value.unwrap(), data).await;
        }
    }

    pub async fn put_from<U: Into<T>>(&self, data: U) {
        self.put(data.into()).await;
    }

    pub async fn take(&self) -> Option<T> {
        let mut lock = self.data.write().await;
        let data = lock.take();
        if data.is_some() {
            self.notify_unset(data.clone().unwrap()).await;
        }
        data
    }

    pub async fn set(&self, data: Option<T>) {
        let mut lock = self.data.write().await;
        let last_value = lock.take();
        *lock = data.clone();
        if last_value.is_some() && data.is_some() {
            self.notify_change(last_value.unwrap(), data.unwrap()).await;
        } else if last_value.is_some() && data.is_none() {
            self.notify_unset(last_value.unwrap()).await;
        } else if data.is_some() && last_value.is_none() {
            self.notify_set(data.unwrap()).await;
        }
    }

    pub async fn set_from<U: Into<Option<T>>>(&self, data: U) {
        self.set(data.into()).await;
    }
    
    pub async fn listen_set(&self, name: &'static str, listener: impl Fn(OptionalStateSetEvent<T>)
        -> Pin<Box<dyn Future<Output=()> + Send + Sync>> + Send + Sync + 'static)
    {
        self.on_set.write().await.push(Box::new(listener));
        self.id_set.write().await.push(name);
    }
    
    pub async fn listen_unset(&self, name: &'static str, listener: impl Fn(OptionalStateUnsetEvent<T>)
        -> Pin<Box<dyn Future<Output=()> + Send + Sync>> + Send + Sync + 'static)
    {
        self.on_unset.write().await.push(Box::new(listener));
        self.id_unset.write().await.push(name);
    }
    
    pub async fn listen_change(&self, name: &'static str, listener: impl Fn(OptionalStateChangedEvent<T>)
        -> Pin<Box<dyn Future<Output=()> + Send + Sync>> + Send + Sync + 'static)
    {
        self.on_change.write().await.push(Box::new(listener));
        self.id_change.write().await.push(name);
    }
    
    pub async fn remove_listener_set(&self, name: &'static str) {
        let mut listeners = self.on_set.write().await;
        let mut ids = self.id_set.write().await;
        let index = ids.iter().position(|id| *id == name).unwrap();
        listeners.remove(index);
        ids.remove(index);
    }
    
    pub async fn remove_listener_unset(&self, name: &'static str) {
        let mut listeners = self.on_unset.write().await;
        let mut ids = self.id_unset.write().await;
        let index = ids.iter().position(|id| *id == name).unwrap();
        listeners.remove(index);
        ids.remove(index);
    }
    
    pub async fn remove_listener_change(&self, name: &'static str) {
        let mut listeners = self.on_change.write().await;
        let mut ids = self.id_change.write().await;
        let index = ids.iter().position(|id| *id == name).unwrap();
        listeners.remove(index);
        ids.remove(index);
    }

    pub async fn notify_set(&self, value: T) {
        let event = OptionalStateSetEvent {
            state: self.clone(),
            gadget: self.gadget.clone(),
            value,
        };
        for listener in self.on_set.read().await.iter() {
            async_runtime().spawn(listener(event.clone()));
        }
    }

    pub async fn notify_change(&self, last_value: T, new_value: T) {
        let event = OptionalStateChangedEvent {
            state: self.clone(),
            gadget: self.gadget.clone(),
            last_value,
            new_value,
        };
        for listener in self.on_change.read().await.iter() {
            async_runtime().spawn(listener(event.clone()));
        }
    }

    pub async fn notify_unset(&self, last_value: T) {
        let event = OptionalStateUnsetEvent {
            state: self.clone(),
            gadget: self.gadget.clone(),
            last_value,
        };
        for listener in self.on_unset.read().await.iter() {
            async_runtime().spawn(listener(event.clone()));
        }
    }

    pub async fn is_set(&self) -> bool {
        self.data.read().await.is_some()
    }
}

pub struct StateVec<T: Send + Sync + Clone> {
    data: Arc<RwLock<Vec<T>>>,
    gadget: GadgetRef,
    on_add: Listeners<StateVecAddEvent<T>>,
    on_set: Listeners<StateVecSetEvent<T>>,
    on_remove: Listeners<StateVecRemoveEvent<T>>,
    id_add: ListenerIdentities,
    id_set: ListenerIdentities,
    id_remove: ListenerIdentities,
}

impl<T: Send + Sync + Clone> Clone for StateVec<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            gadget: self.gadget.clone(),
            on_add: self.on_add.clone(),
            on_set: self.on_set.clone(),
            on_remove: self.on_remove.clone(),
            id_add: self.id_add.clone(),
            id_set: self.id_set.clone(),
            id_remove: self.id_remove.clone(),
        }
    }
}

impl<T: Send + Sync + Clone> Default for StateVec<T> {
    fn default() -> Self {
        Self {
            data: Arc::new(RwLock::new(Vec::new())),
            gadget: GadgetRef::default(),
            on_add: Arc::new(RwLock::new(Vec::new())),
            on_set: Arc::new(RwLock::new(Vec::new())),
            on_remove: Arc::new(RwLock::new(Vec::new())),
            id_add: Arc::new(RwLock::new(Vec::new())),
            id_set: Arc::new(RwLock::new(Vec::new())),
            id_remove: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

pub struct StateVecAddEvent<T: Send + Sync + Clone> {
    pub state: StateVec<T>,
    pub gadget: GadgetRef,
    pub index: usize,
    pub new_value: T,
}

impl<T: Send + Sync + Clone> Clone for StateVecAddEvent<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            gadget: self.gadget.clone(),
            index: self.index,
            new_value: self.new_value.clone(),
        }
    }
}

pub struct StateVecSetEvent<T: Send + Sync + Clone> {
    pub state: StateVec<T>,
    pub gadget: GadgetRef,
    pub index: usize,
    pub old_value: T,
    pub new_value: T,
}

impl<T: Send + Sync + Clone> Clone for StateVecSetEvent<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            gadget: self.gadget.clone(),
            index: self.index,
            old_value: self.old_value.clone(),
            new_value: self.new_value.clone(),
        }
    }
}

pub struct StateVecRemoveEvent<T: Send + Sync + Clone> {
    pub state: StateVec<T>,
    pub gadget: GadgetRef,
    pub old_index: usize,
    pub old_value: T,
}

impl<T: Send + Sync + Clone> Clone for StateVecRemoveEvent<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            gadget: self.gadget.clone(),
            old_index: self.old_index,
            old_value: self.old_value.clone(),
        }
    }
}

impl<T: Send + Sync + Clone> StateVec<T> {
    pub fn new(gadget: GadgetRef) -> Self {
        Self {
            data: Arc::new(RwLock::new(Vec::new())),
            gadget,
            on_add: Arc::new(Default::default()),
            on_set: Arc::new(Default::default()),
            on_remove: Arc::new(Default::default()),
            id_add: Arc::new(Default::default()),
            id_set: Arc::new(Default::default()),
            id_remove: Arc::new(Default::default())
        }
    }

    pub async fn get_vec(&self) -> RwLockReadGuard<'_, Vec<T>> {
        self.data.read().await
    }

    pub async fn get_vec_mut(&self) -> RwLockWriteGuard<'_, Vec<T>> {
        self.data.write().await
    }

    pub async fn push(&self, data: T) {
        let mut lock = self.data.write().await;
        lock.push(data.clone());
        self.notify_add(lock.len() - 1, data).await;
    }

    pub async fn push_from<U: Into<T>>(self: &Self, data: U) {
        self.push(data.into()).await;
    }

    pub async fn pop(&self) -> Option<T> {
        let mut lock = self.data.write().await;
        let result = lock.pop();
        if let Some(value) = result.clone() {
            self.notify_remove(lock.len(), value).await;
        }
        result
    }

    pub async fn get(&self, index: usize) -> Option<T> {
        let lock = self.data.read().await;
        lock.get(index).cloned()
    }

    pub async fn set(&self, index: usize, data: T) {
        let mut lock = self.data.write().await;
        let old_value = mem::replace(&mut lock[index], data.clone());
        self.notify_set(index, old_value, data).await;
    }

    pub async fn set_from<U: Into<T>>(self: &Self, index: usize, data: U) {
        self.set(index, data.into()).await;
    }

    pub async fn remove_at(&self, index: usize) -> T {
        let mut lock = self.data.write().await;
        let old_value = lock.remove(index);
        self.notify_remove(index, old_value.clone()).await;
        old_value
    }

    pub async fn remove(&self, data: &T) -> Option<T> where T: PartialEq {
        let mut lock = self.data.write().await;
        let index = lock.iter().position(|x| x == data)?;
        let old_value = lock.remove(index);
        self.notify_remove(index, old_value.clone()).await;
        Some(old_value)
    }

    pub async fn listen_add(&self, name: &'static str, listener: impl Fn(StateVecAddEvent<T>)
        -> Pin<Box<dyn Future<Output=()> + Send + Sync>> + Send + Sync + 'static)
    {
        self.on_add.write().await.push(Box::new(listener));
        self.id_add.write().await.push(name);
    }

    pub async fn listen_set(&self, name: &'static str, listener: impl Fn(StateVecSetEvent<T>)
        -> Pin<Box<dyn Future<Output=()> + Send + Sync>> + Send + Sync + 'static)
    {
        self.on_set.write().await.push(Box::new(listener));
        self.id_set.write().await.push(name);
    }

    pub async fn listen_remove(&self, name: &'static str, listener: impl Fn(StateVecRemoveEvent<T>)
        -> Pin<Box<dyn Future<Output=()> + Send + Sync>> + Send + Sync + 'static)
    {
        self.on_remove.write().await.push(Box::new(listener));
        self.id_remove.write().await.push(name);
    }
    
    pub async fn remove_listener_add(&self, name: &'static str) {
        let mut on_add = self.on_add.write().await;
        let mut id_on_add = self.id_add.write().await;
        if let Some(index) = id_on_add.iter().position(|x| *x == name) {
            on_add.remove(index);
            id_on_add.remove(index);
        }
    }
    
    pub async fn remove_listener_set(&self, name: &'static str) {
        let mut on_set = self.on_set.write().await;
        let mut id_on_set = self.id_set.write().await;
        if let Some(index) = id_on_set.iter().position(|x| *x == name) {
            on_set.remove(index);
            id_on_set.remove(index);
        }
    }
    
    pub async fn remove_listener_remove(&self, name: &'static str) {
        let mut on_remove = self.on_remove.write().await;
        let mut id_on_remove = self.id_remove.write().await;
        if let Some(index) = id_on_remove.iter().position(|x| *x == name) {
            on_remove.remove(index);
            id_on_remove.remove(index);
        }
    }

    pub async fn notify_add(&self, index: usize, new_value: T) {
        let event = StateVecAddEvent {
            state: self.clone(),
            gadget: self.gadget.clone(),
            index,
            new_value,
        };
        for listener in self.on_add.read().await.iter() {
            async_runtime().spawn(listener(event.clone()));
        }
    }

    pub async fn notify_set(&self, index: usize, old_value: T, new_value: T) {
        let event = StateVecSetEvent {
            state: self.clone(),
            gadget: self.gadget.clone(),
            index,
            old_value,
            new_value,
        };
        for listener in self.on_set.read().await.iter() {
            async_runtime().spawn(listener(event.clone()));
        }
    }

    pub async fn notify_remove(&self, index: usize, old_value: T) {
        let event = StateVecRemoveEvent {
            state: self.clone(),
            gadget: self.gadget.clone(),
            old_index: index,
            old_value,
        };
        for listener in self.on_remove.read().await.iter() {
            async_runtime().spawn(listener(event.clone()));
        }
    }

    pub async fn clear(&self) {
        let mut lock = self.data.write().await;
        while let Some(value) = lock.pop() {
            self.notify_remove(lock.len(), value).await;
        }
    }
}

pub struct StateMap<K: Send + Sync + Clone + Eq + Hash, V: Send + Sync + Clone> {
    data: Arc<RwLock<HashMap<K, V>>>,
    gadget: GadgetRef,
    listeners: Listeners<StateMapEvent<K, V>>,
    identities: ListenerIdentities,
}

impl<K: Send + Sync + Clone + Eq + Hash, V: Send + Sync + Clone>
Clone for StateMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            gadget: self.gadget.clone(),
            listeners: self.listeners.clone(),
            identities: self.identities.clone(),
        }
    }
}

impl<K: Send + Sync + Clone + Eq + Hash, V: Send + Sync + Clone>
Default for StateMap<K, V> {
    fn default() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            gadget: GadgetRef::default(),
            listeners: Default::default(),
            identities: Default::default(),
        }
    }
}

pub struct StateMapEvent<K: Send + Sync + Clone + Eq + Hash, V: Send + Sync + Clone> {
    pub state: StateMap<K, V>,
    pub gadget: GadgetRef,
    pub key: K,
    pub old_value: Option<V>,
    pub new_value: Option<V>,
}

impl<K: Send + Sync + Clone + Eq + Hash, V: Send + Sync + Clone>
Clone for StateMapEvent<K, V> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            gadget: self.gadget.clone(),
            key: self.key.clone(),
            old_value: self.old_value.clone(),
            new_value: self.new_value.clone(),
        }
    }
}

impl<K: Send + Sync + Clone + Eq + Hash, V: Send + Sync + Clone>
StateMap<K, V> {
    pub fn new(gadget: GadgetRef) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            gadget,
            listeners: Default::default(),
            identities: Arc::new(Default::default())
        }
    }

    pub async fn get_map(&self) -> RwLockReadGuard<'_, HashMap<K, V>> {
        self.data.read().await
    }

    pub async fn get_map_mut(&self) -> RwLockWriteGuard<'_, HashMap<K, V>> {
        self.data.write().await
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let lock = self.data.read().await;
        lock.get(key).cloned()
    }

    pub async fn set(&self, key: K, value: V) {
        let mut lock = self.data.write().await;
        let old_value = lock.insert(key.clone(), value.clone());
        self.notify(key, old_value, Some(value)).await;
    }

    pub async fn set_from<U: Into<V>>(self: &Self, key: K, value: U) {
        self.set(key, value.into()).await;
    }

    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut lock = self.data.write().await;
        let old_value = lock.remove(key);
        if let Some(value) = old_value.clone() {
            self.notify(key.clone(), Some(value), None).await;
        }
        old_value
    }

    pub async fn listen(&self, name: &'static str, listener: impl Fn(StateMapEvent<K, V>)
        -> Pin<Box<dyn Future<Output=()> + Send + Sync>> + Send + Sync + 'static)
    {
        self.listeners.write().await.push(Box::new(listener));
        self.identities.write().await.push(name);
    }
    
    pub async fn remove_listener(&self, name: &'static str) {
        let mut listeners = self.listeners.write().await;
        let mut identities = self.identities.write().await;
        if let Some(index) = identities.iter().position(|x| *x == name) {
            listeners.remove(index);
            identities.remove(index);
        }
    }

    pub async fn notify(&self, key: K, old_value: Option<V>, new_value: Option<V>) {
        let event = StateMapEvent {
            state: self.clone(),
            gadget: self.gadget.clone(),
            key,
            old_value,
            new_value,
        };
        for listener in self.listeners.read().await.iter() {
            async_runtime().spawn(listener(event.clone()));
        }
    }
}