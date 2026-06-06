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
    use core::fmt;
    use std::sync::{Arc, Mutex};
    use tracing::{
        dispatcher::{with_default, Dispatch},
        field::{Field, Visit},
        Event, Subscriber,
    };
    use tracing_subscriber::{
        filter::LevelFilter,
        layer::{Context, SubscriberExt},
        registry::{LookupSpan, Registry},
        Layer,
    };

    #[derive(Clone)]
    struct CapturedEvent {
        level: Level,
        target: String,
        message: String,
    }

    #[derive(Clone)]
    struct CaptureLayer {
        events: Arc<Mutex<Vec<CapturedEvent>>>,
    }

    impl<S> Layer<S> for CaptureLayer
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
            let mut visitor = MessageVisitor::default();
            event.record(&mut visitor);
            self.events.lock().unwrap().push(CapturedEvent {
                level: *event.metadata().level(),
                target: event.metadata().target().to_string(),
                message: visitor.message.unwrap_or_default(),
            });
        }
    }

    #[derive(Default)]
    struct MessageVisitor {
        message: Option<String>,
    }

    impl Visit for MessageVisitor {
        fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
            if field.name() == "message" {
                self.message = Some(format!("{value:?}"));
            }
        }
    }

    #[test]
    fn we_can_log_memory_usage_when_trace_is_enabled() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let subscriber = Registry::default()
            .with(LevelFilter::TRACE)
            .with(CaptureLayer {
                events: Arc::clone(&events),
            });
        let dispatch = Dispatch::new(subscriber);

        with_default(&dispatch, || log_memory_usage("test"));

        let events = events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].level, Level::TRACE);
        assert!(events[0].target.ends_with("utils::log"));
        assert!(events[0].message.contains("test Available memory"));
    }
}
