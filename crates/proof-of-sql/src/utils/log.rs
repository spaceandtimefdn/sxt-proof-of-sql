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

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::log_memory_usage;
    use core::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tracing::subscriber::Interest;
    use tracing::{
        span::{Attributes, Id, Record},
        Event, Metadata, Subscriber,
    };

    struct TraceSubscriber {
        events: Arc<AtomicUsize>,
    }

    impl Subscriber for TraceSubscriber {
        fn enabled(&self, metadata: &Metadata<'_>) -> bool {
            *metadata.level() == tracing::Level::TRACE
        }

        fn new_span(&self, _span: &Attributes<'_>) -> Id {
            Id::from_u64(1)
        }

        fn record(&self, _span: &Id, _values: &Record<'_>) {}

        fn record_follows_from(&self, _span: &Id, _follows: &Id) {}

        fn event(&self, event: &Event<'_>) {
            if *event.metadata().level() == tracing::Level::TRACE {
                self.events.fetch_add(1, Ordering::SeqCst);
            }
        }

        fn enter(&self, _span: &Id) {}

        fn exit(&self, _span: &Id) {}

        fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest {
            if *metadata.level() == tracing::Level::TRACE {
                Interest::always()
            } else {
                Interest::never()
            }
        }
    }

    #[test]
    fn log_memory_usage_emits_trace_event_when_trace_is_enabled() {
        let events = Arc::new(AtomicUsize::new(0));
        let subscriber = TraceSubscriber {
            events: Arc::clone(&events),
        };

        tracing::subscriber::with_default(subscriber, || {
            log_memory_usage("coverage-marker");
        });

        assert_eq!(events.load(Ordering::SeqCst), 1);
    }
}
