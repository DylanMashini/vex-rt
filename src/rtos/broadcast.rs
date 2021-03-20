use alloc::sync::{Arc, Weak};
use owner_monad::{Owner, OwnerMut};

use super::{handle_event, Event, EventHandle, GenericSleep, Mutex, Selectable};
use crate::error::Error;

/// Represents a source of data which notifies listeners on a new value.
pub struct Broadcast<T: Clone>(Mutex<BroadcastData<T>>);

impl<T: Clone> Broadcast<T> {
    #[inline]
    /// Creates a new broadcast event with the associated initial value. Panics
    /// on failure; see [`Broadcast::try_new()`].
    pub fn new(data: T) -> Self {
        Self::try_new(data).unwrap_or_else(|err| panic!("failed to create broadcast: {:?}", err))
    }

    /// Creates a new broadcast event with the associated initial value.
    pub fn try_new(data: T) -> Result<Self, Error> {
        Ok(Self(Mutex::try_new(BroadcastData {
            data: Arc::new(data),
            event: Event::new(),
        })?))
    }

    /// Gets a copy of the current value of the broadcast event.
    pub fn value(&self) -> T {
        (*self.0.lock().data).clone()
    }

    #[inline]
    /// Creates a new listener for the broadcast event.
    pub fn listen(&self) -> BroadcastListener<'_, T> {
        BroadcastListener::new(Weak::new(), &self.0)
    }

    /// Publishes a new value for the broadcast event.
    pub fn publish(&self, data: T) {
        let mut lock = self.0.lock();
        lock.data = Arc::new(data);
        lock.event.notify();
    }
}

/// Provides a means of listening to updates from a [`Broadcast`] event.
pub struct BroadcastListener<'a, T: Clone> {
    data: Weak<T>,
    mtx: &'a Mutex<BroadcastData<T>>,
}

impl<'a, T: Clone> BroadcastListener<'a, T> {
    #[inline]
    fn new(data: Weak<T>, mtx: &'a Mutex<BroadcastData<T>>) -> Self {
        Self { data, mtx }
    }

    #[inline]
    /// Get the latest unprocessed value from the event, if there is one.
    pub fn next_value(&mut self) -> Option<T> {
        Self::next_value_impl(&mut self.data, &self.mtx)
    }

    #[inline]
    /// A [`Selectable`] event which occurs when new data is published to the
    /// underlying [`Broadcast`] event.
    pub fn select(&'_ mut self) -> impl Selectable<Result = T> + '_ {
        struct BroadcastSelect<'b, T> {
            data: &'b mut Weak<T>,
            mtx: &'b Mutex<BroadcastData<T>>,
        }

        struct BroadcastEvent<'b, T: Clone> {
            data: &'b mut Weak<T>,
            handle: EventHandle<&'b Mutex<BroadcastData<T>>>,
        }

        impl<'b, T: Clone> Selectable for BroadcastSelect<'b, T> {
            const COUNT: u32 = 1;

            type Result = T;

            type Event = BroadcastEvent<'b, T>;

            #[inline]
            fn listen(self, offset: u32) -> Self::Event {
                BroadcastEvent {
                    data: self.data,
                    handle: handle_event(self.mtx, offset),
                }
            }

            #[inline]
            fn poll(mut event: Self::Event, _mask: u32) -> Result<T, Self::Event> {
                let data = &mut event.data;
                event
                    .handle
                    .with(|mtx| BroadcastListener::next_value_impl(data, &mtx))
                    .flatten()
                    .ok_or(event)
            }

            #[inline]
            fn sleep(event: &Self::Event) -> GenericSleep {
                GenericSleep::NotifyTake(None)
            }
        }

        BroadcastSelect {
            data: &mut self.data,
            mtx: self.mtx,
        }
    }

    fn next_value_impl(data: &mut Weak<T>, mtx: &'a Mutex<BroadcastData<T>>) -> Option<T> {
        let lock = mtx.lock();
        match data.upgrade() {
            Some(arc) if Arc::ptr_eq(&arc, &lock.data) => None,
            _ => {
                *data = Arc::downgrade(&lock.data);
                Some((*lock.data).clone())
            }
        }
    }
}

impl<T> OwnerMut<Event> for &Mutex<BroadcastData<T>> {
    fn with<'a, U>(&'a mut self, f: impl FnOnce(&mut Event) -> U) -> Option<U>
    where
        Event: 'a,
    {
        Some(f(&mut self.try_lock().ok()?.event))
    }
}

struct BroadcastData<T> {
    data: Arc<T>,
    event: Event,
}
