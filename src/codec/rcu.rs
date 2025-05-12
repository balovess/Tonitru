use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::mem; // Import the mem module

/// A basic Read-Copy-Update (RCU) prototype for lock-free reads.
///
/// This implementation allows multiple readers to access the data concurrently
/// with a single writer. Reads are lock-free. Updates involve copying the data
/// and atomically swapping a pointer.
///
/// NOTE: This is a simplified prototype and does NOT include a grace period
/// for safe memory reclamation of old data versions. In a real-world RCU,
/// a mechanism is needed to ensure all readers of an old version have finished
/// before that version's memory is deallocated.
pub struct Rcu<T> {
    // Atomic pointer to the current data.
    // We use a raw pointer inside AtomicPtr because AtomicPtr works with raw pointers.
    // The data itself is managed by Arc for shared ownership.
    data: AtomicPtr<T>,
}

// Safety: Rcu is Send and Sync if T is Send and Sync.
// Readers only get a shared reference (&T), writers use atomic operations.
unsafe impl<T: Send + Sync> Send for Rcu<T> {}
unsafe impl<T: Send + Sync> Sync for Rcu<T> {}

impl<T> Rcu<T> {
    /// Creates a new RCU instance with initial data.
    pub fn new(data: T) -> Self {
        // Allocate data on the heap and wrap in Arc for shared ownership.
        // Then get a raw pointer to the Arc'd data.
        let arc_data = Arc::new(data);
        let raw_ptr = Arc::into_raw(arc_data) as *mut T;

        Rcu {
            data: AtomicPtr::new(raw_ptr),
        }
    }

    /// Reads the current data. This operation is lock-free.
    ///
    /// Returns an `Arc` to the data, ensuring the data remains valid
    /// as long as the `Arc` is held.
    pub fn read(&self) -> Arc<T> {
        // Atomically load the current pointer.
        let raw_ptr = self.data.load(Ordering::Acquire);

        // Convert the raw pointer back to an Arc.
        // This increments the Arc's reference count.
        // We use `from_raw` and then `clone` to get a new Arc that we return,
        // while the original Arc (represented by the loaded raw_ptr) is
        // immediately forgotten to avoid decrementing its ref count here.
        let arc_data = unsafe { Arc::from_raw(raw_ptr) };
        let cloned_arc = Arc::clone(&arc_data);
        mem::forget(arc_data); // Prevent decrementing ref count

        cloned_arc
    }

    /// Updates the data. This involves creating a new copy and atomically
    /// swapping the pointer.
    ///
    /// NOTE: This prototype does NOT handle the safe deallocation of the
    /// old data version.
    pub fn update(&self, new_data: T) {
        // Create a new Arc for the new data.
        let new_arc_data = Arc::new(new_data);
        let new_raw_ptr = Arc::into_raw(new_arc_data) as *mut T;

        // Atomically swap the pointer.
        // The old raw pointer is returned.
        let old_raw_ptr = self.data.swap(new_raw_ptr, Ordering::Release);

        // In a real RCU, the old_raw_ptr would be passed to a grace period
        // mechanism for eventual deallocation. In this prototype, we just
        // forget it, which will lead to a memory leak for the old data.
        // DO NOT use this RCU prototype in production where updates occur!
        let old_arc_data = unsafe { Arc::from_raw(old_raw_ptr) };
        mem::forget(old_arc_data); // Leak the old Arc
    }

    // TODO: Implement a proper grace period mechanism for safe memory reclamation.
}

impl<T> Drop for Rcu<T> {
    fn drop(&mut self) {
        // When the Rcu itself is dropped, we need to deallocate the currently
        // held data. This is safe because the Rcu is being dropped, meaning
        // no new readers can acquire the pointer. However, existing readers
        // holding Arcs to this data will still keep it alive until their Arcs are dropped.
        let raw_ptr = self.data.load(Ordering::Acquire);
        if !raw_ptr.is_null() {
            let _arc_data = unsafe { Arc::from_raw(raw_ptr) }; // Added underscore
            // Arc will be dropped here, deallocating the data if it's the last reference.
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_rcu_basic() {
        let rcu = Rcu::new(10);

        // Initial read
        let data1 = rcu.read();
        assert_eq!(*data1, 10);

        // Update data
        rcu.update(20);

        // Read after update (should see new data)
        let data2 = rcu.read();
        assert_eq!(*data2, 20);

        // Old read should still see old data (due to Arc, but memory is leaked in this proto)
        // In a real RCU, data1 would still be valid until a grace period passes.
        // With this prototype's leak, data1 remains valid but its memory isn't reclaimed.
        assert_eq!(*data1, 10);
    }

    #[test]
    fn test_rcu_concurrent_reads() {
        let rcu = Arc::new(Rcu::new(100));
        let mut handles = vec![];

        // Create multiple reader threads
        for i in 0..5 {
            let rcu_clone = Arc::clone(&rcu);
            handles.push(thread::spawn(move || {
                // Perform multiple reads
                for _ in 0..10 {
                    let data = rcu_clone.read();
                    println!("Reader {} read: {}", i, *data);
                    // In a real RCU, we'd assert that the data is one of the valid versions.
                    // Here, we just check it's either 100 or 200.
                    assert!(*data == 100 || *data == 200);
                    thread::sleep(Duration::from_millis(10));
                }
            }));
        }

        // Allow some reads to happen before updating
        thread::sleep(Duration::from_millis(50));

        // Update data from the main thread
        println!("Main thread updating data to 200");
        rcu.update(200);

        // Allow more reads to happen
        thread::sleep(Duration::from_millis(50));

        // Update data again (this will leak the 200 data)
        println!("Main thread updating data to 300 (will leak 200)");
        rcu.update(300); // This update will leak the previous version (200)

        // Wait for reader threads to finish
        for handle in handles {
            handle.join().unwrap();
        }

        // Final read from main thread
        let final_data = rcu.read();
        assert_eq!(*final_data, 300);

        // Note: This test demonstrates concurrent reads and updates, but
        // the memory for the '100' and '200' data versions is leaked
        // because the prototype lacks a grace period and safe reclamation.
    }
}
