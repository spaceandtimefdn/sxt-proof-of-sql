use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
#[cfg(feature = "std")]
use sysinfo::System;
use tracing::{trace, Level};

// Static counters for allocation tracking
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
    pub fn report() -> (usize, usize) {
        let count = ALLOCATION_COUNT.load(Ordering::SeqCst);
        let bytes = ALLOCATION_BYTES.load(Ordering::SeqCst);
        trace!("Total allocations: {} with {} bytes", count, bytes);
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
