use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
#[cfg(feature = "std")]
use sysinfo::System;
use tracing::{trace, Level};

// Static counters for allocation tracking
//
// Using static atomic counters allows us to:
// 1. Track allocations across the entire program without passing tracker instances
// 2. Provide zero-cost tracking when not enabled (no allocation overhead for tracker objects)
// 3. Maintain thread safety through atomic operations
// 4. Support global reset/reporting through a simple API
//
// Note: This approach creates global mutable state, which means tests must be
// careful about resetting between test cases. For testing, we run tests
// sequentially to avoid interference between concurrent tests.
static ALLOCATION_COUNT: AtomicUsize = AtomicUsize::new(0);
static ALLOCATION_BYTES: AtomicUsize = AtomicUsize::new(0);

/// Tracks memory allocations in the codebase
pub struct AllocationTracker;

impl AllocationTracker {
    /// Reset the allocation counters
    pub fn reset() {
        ALLOCATION_COUNT.store(0, Ordering::SeqCst);
        ALLOCATION_BYTES.store(0, Ordering::SeqCst);
    }

    /// Record a generic allocation with type information
    pub fn record_allocation<T>(name: &str, count: usize, capacity: usize) {
        let alloc_count = ALLOCATION_COUNT.fetch_add(1, Ordering::SeqCst);
        let bytes = capacity * core::mem::size_of::<T>();
        let total_bytes = ALLOCATION_BYTES.fetch_add(bytes, Ordering::SeqCst);

        trace!(
            "Allocation #{}: {} of {} elements with a capacity of {} of type {} ({} bytes), total: {} bytes", 
            alloc_count + 1,
            name,
            count,
            capacity,
            core::any::type_name::<T>(),
            bytes,
            total_bytes + bytes
        );
    }

    /// Record an allocation for a vector
    pub fn record_vec<T>(name: &str, vec: &Vec<T>) {
        Self::record_allocation::<T>(name, vec.len(), vec.capacity());
    }

    /// Get the current allocation statistics
    #[expect(clippy::cast_precision_loss)]
    pub fn report() -> (usize, usize) {
        let count = ALLOCATION_COUNT.load(Ordering::SeqCst);
        let bytes = ALLOCATION_BYTES.load(Ordering::SeqCst);
        let megabytes = bytes as f64 / (1024.0 * 1024.0);
        trace!("Total allocations: {} with {:.2} MB", count, megabytes);
        (count, bytes)
    }
}

/// Starts memory and allocation tracking at the TRACE level.
pub fn start() {
    if tracing::level_enabled!(Level::TRACE) {
        AllocationTracker::reset();
        log_memory_usage("Start");
    }
}

/// Stops memory and allocation tracking.
pub fn stop() {
    if tracing::level_enabled!(Level::TRACE) {
        log_memory_usage("Stop");
        AllocationTracker::report();
    }
}

/// Logs the memory usage of the system at the TRACE level.
///
/// This function logs the available memory, used memory, and the percentage of memory used.
/// It only logs this information if the TRACE level is enabled in the tracing configuration.
///
/// # Arguments
///
/// * `name` - A string slice that holds the name to be included in the log message.
#[expect(clippy::cast_precision_loss)]
pub fn log_memory_usage(name: &str) {
    #[cfg(feature = "std")]
    if tracing::level_enabled!(Level::TRACE) {
        let mut system = System::new_all();
        system.refresh_memory();

        let available_memory = system.available_memory() as f64 / (1024.0 * 1024.0);
        let used_memory = system.used_memory() as f64 / (1024.0 * 1024.0);
        let percentage_memory_used = (used_memory / (used_memory + available_memory)) * 100.0;

        trace!(
            "{} Available memory: {:.2} MB, Used memory: {:.2} MB, Percentage memory used: {:.2}%",
            name,
            available_memory,
            used_memory,
            percentage_memory_used
        );
    }
}

/// Logs detailed information about a vector allocation
///
/// # Arguments
///
/// * `name` - A descriptive name for the vector
/// * `vec` - The vector to log information about
pub fn log_vector<T>(name: &str, vec: &Vec<T>) {
    if tracing::level_enabled!(Level::TRACE) {
        AllocationTracker::record_vec(name, vec);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn we_can_reset_allocation_tracking() {
        // Make some allocations first
        let v: Vec<u32> = vec![1, 2, 3, 4];
        AllocationTracker::record_vec("test_vec", &v);

        // Reset and verify counters are zeroed
        AllocationTracker::reset();
        let (count, bytes) = AllocationTracker::report();
        assert_eq!(count, 0, "Counter should be reset to zero");
        assert_eq!(bytes, 0, "Bytes should be reset to zero");
    }

    fn we_can_count_allocations() {
        AllocationTracker::reset();

        // Record some allocations
        AllocationTracker::record_allocation::<u32>("test1", 10, 16);
        AllocationTracker::record_allocation::<u32>("test2", 5, 8);

        // Check the count
        let (count, _) = AllocationTracker::report();
        assert_eq!(count, 2, "Should have recorded 2 allocations");
    }

    fn we_can_calcuate_allocation_bytes() {
        AllocationTracker::reset();

        // u32 = 4 bytes
        // 16 × 4 = 64 bytes
        AllocationTracker::record_allocation::<u32>("test", 10, 16);

        // Check bytes
        let (_, bytes) = AllocationTracker::report();
        assert_eq!(bytes, 64, "Should have recorded 64 bytes for 16 u32s");
    }

    fn we_can_allocate_track_a_vec() {
        AllocationTracker::reset();

        // Create a vector with known capacity
        let mut v = Vec::with_capacity(10);
        v.push(1u32);
        v.push(2u32);
        v.push(3u32);

        // Record it
        AllocationTracker::record_vec("test_vec", &v);

        // Check results
        let (count, bytes) = AllocationTracker::report();
        assert_eq!(count, 1, "Should have recorded 1 allocation");
        assert_eq!(bytes, 40, "Should have recorded 40 bytes (10 u32s)");
    }

    fn we_can_track_allocations_of_different_types_of_vecs() {
        AllocationTracker::reset();

        // u8 = 1 byte
        AllocationTracker::record_allocation::<u8>("u8_test", 10, 10);
        let (_, bytes_u8) = AllocationTracker::report();
        assert_eq!(bytes_u8, 10, "Should record 10 bytes for 10 u8s");

        AllocationTracker::reset();

        // u64 = 8 bytes
        AllocationTracker::record_allocation::<u64>("u64_test", 10, 10);
        let (_, bytes_u64) = AllocationTracker::report();
        assert_eq!(bytes_u64, 80, "Should record 80 bytes for 10 u64s");
    }

    // Test sequentially to ensure we don't clear the static
    // values during the parallel execution of tests.
    #[test]
    fn we_can_perform_allocation_tracking() {
        we_can_reset_allocation_tracking();
        we_can_count_allocations();
        we_can_calcuate_allocation_bytes();
        we_can_allocate_track_a_vec();
        we_can_track_allocations_of_different_types_of_vecs();
    }
}
