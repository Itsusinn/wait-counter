use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;

struct Inner {
    count: AtomicUsize,
    notify: Notify,
}

pub struct WaitCounter {
    inner: Arc<Inner>,
}

impl WaitCounter {
    pub fn new() -> Self {
        let inner = Inner {
            count: AtomicUsize::new(1),
            notify: Notify::new(),
        };
        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn wake_clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

    pub async fn wait(&self) {
        loop {
            // Use Acquire ordering to ensure the latest write is seen
            let current = self.inner.count.load(Ordering::Acquire);
            if current == 1 {
                break;
            }
            // Waiting for notification, may cause false wakeup, so loop checking is required
            self.inner.notify.notified().await;
        }
    }
}

impl Clone for WaitCounter {
    fn clone(&self) -> Self {
        // Use Relaxed ordering because no other memory synchronization is required here
        self.inner.count.fetch_add(1, Ordering::Relaxed);
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Drop for WaitCounter {
    fn drop(&mut self) {
        // Use Release ordering to ensure previous writes complete before decrementing count
        let prev = self.inner.count.fetch_sub(1, Ordering::Release);
        // When the previous value is 2, it decreases to 1, triggering a notification
        if prev == 2 {
            self.inner.notify.notify_waiters();
        }
    }
}
#[cfg(test)]
mod test {
    use crate::WaitCounter;
    use std::time::Duration;

    #[tokio::test]
    async fn test_wait_counter() {
        let counter = WaitCounter::new();
        let cloned = counter.clone();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(1000)).await;
            drop(cloned);
        });

        counter.wait().await;
        println!("Counter reached 1");
    }
}
