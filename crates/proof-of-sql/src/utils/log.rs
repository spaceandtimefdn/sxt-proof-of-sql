#[cfg(feature = "std")]
use sysinfo::System;
use tracing::{trace, Level};

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

#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tracing::{
        span::{Attributes, Id, Record},
        Event, Metadata, Subscriber,
    };

    struct TraceCountingSubscriber {
        events: AtomicUsize,
    }

    impl TraceCountingSubscriber {
        const fn new() -> Self {
            Self {
                events: AtomicUsize::new(0),
            }
        }

        fn event_count(&self) -> usize {
            self.events.load(Ordering::Relaxed)
        }
    }

    impl Subscriber for TraceCountingSubscriber {
        fn enabled(&self, metadata: &Metadata<'_>) -> bool {
            *metadata.level() == Level::TRACE
        }

        fn new_span(&self, _span: &Attributes<'_>) -> Id {
            Id::from_u64(1)
        }

        fn record(&self, _span: &Id, _values: &Record<'_>) {}

        fn record_follows_from(&self, _span: &Id, _follows: &Id) {}

        fn event(&self, _event: &Event<'_>) {
            self.events.fetch_add(1, Ordering::Relaxed);
        }

        fn enter(&self, _span: &Id) {}

        fn exit(&self, _span: &Id) {}
    }

    #[test]
    fn we_skip_memory_logging_when_trace_is_disabled() {
        log_memory_usage("trace-disabled");
    }

    #[test]
    fn we_emit_memory_logging_when_trace_is_enabled() {
        let subscriber = Arc::new(TraceCountingSubscriber::new());
        let observed_subscriber = Arc::clone(&subscriber);

        tracing::subscriber::with_default(subscriber, || {
            log_memory_usage("trace-enabled");
        });

        assert_eq!(observed_subscriber.event_count(), 1);
    }
}
