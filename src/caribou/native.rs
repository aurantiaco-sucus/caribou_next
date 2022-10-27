use std::fmt::{Debug, Formatter, Pointer};
use std::any::{Any};
use std::sync::Arc;

pub struct Native {
    data: Arc<dyn Wrapper>
}

impl Debug for Native {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

impl PartialEq for Native {
    fn eq(&self, other: &Self) -> bool {
        #[allow(clippy::vtable_address_comparisons)]
        Arc::ptr_eq(&self.data, &other.data)
    }
}

impl Clone for Native {
    fn clone(&self) -> Self {
        Native {
            data: self.data.clone()
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct NativePlaceholder;

impl Wrapper for NativePlaceholder {
    fn debug_fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }

    fn get(&self) -> Box<dyn Any> {
        Box::new(NativePlaceholder)
    }
}

impl Default for Native {
    fn default() -> Self {
        Native {
            data: Arc::new(NativePlaceholder)
        }
    }
}

impl Native {
    pub fn wrap<T: 'static + Wrapper>(data: T) -> Self {
        Self { data: Arc::new(data) }
    }

    pub fn get<T: 'static + Any>(&self) -> Result<Box<T>, Box<(dyn Any + 'static)>> {
        self.data.get().downcast::<T>()
    }

    pub fn is_placeholder(&self) -> bool {
        self.data.get().is::<NativePlaceholder>()
    }
}

pub trait Wrapper: Send + Sync {
    fn debug_fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
    fn get(&self) -> Box<dyn Any>;
}
