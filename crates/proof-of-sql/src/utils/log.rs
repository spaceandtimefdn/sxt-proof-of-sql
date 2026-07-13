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
    use super::*;
    use tracing::{
        metadata::LevelFilter,
        span::{Attributes, Id, Record},
        subscriber::Interest,
        Event, Metadata, Subscriber,
    };

    struct TraceSubscriber;

    impl Subscriber for TraceSubscriber {
        fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
            true
        }

        fn new_span(&self, _span: &Attributes<'_>) -> Id {
            Id::from_u64(1)
        }

        fn record(&self, _span: &Id, _values: &Record<'_>) {}

        fn record_follows_from(&self, _span: &Id, _follows: &Id) {}

        fn event(&self, _event: &Event<'_>) {}

        fn enter(&self, _span: &Id) {}

        fn exit(&self, _span: &Id) {}

        fn register_callsite(&self, _metadata: &'static Metadata<'static>) -> Interest {
            Interest::always()
        }

        fn max_level_hint(&self) -> Option<LevelFilter> {
            Some(LevelFilter::TRACE)
        }
    }

    #[test]
    fn log_memory_usage_runs_when_trace_is_enabled() {
        tracing::subscriber::with_default(TraceSubscriber, || log_memory_usage("coverage"));
    }
}
