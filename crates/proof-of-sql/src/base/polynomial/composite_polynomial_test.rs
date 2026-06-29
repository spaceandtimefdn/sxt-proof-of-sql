use super::CompositePolynomial;
use crate::base::scalar::test_scalar::TestScalar;
use alloc::rc::Rc;
use std::sync::{Arc, Mutex};
use tracing::{
    field::{Field, Visit},
    span::{Attributes, Id, Record},
    Event, Metadata, Subscriber,
};

#[derive(Default)]
struct ProductTraceSubscriber {
    events: Arc<Mutex<Vec<String>>>,
}

impl Subscriber for ProductTraceSubscriber {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.target() == "proof_of_sql::base::polynomial::composite_polynomial"
    }

    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(1)
    }

    fn record(&self, _: &Id, _: &Record<'_>) {}

    fn record_follows_from(&self, _: &Id, _: &Id) {}

    fn event(&self, event: &Event<'_>) {
        if event.metadata().level() == &tracing::Level::INFO {
            let mut visitor = MessageVisitor::default();
            event.record(&mut visitor);
            if visitor.message.starts_with("Product #") {
                self.events.lock().unwrap().push(visitor.message);
            }
        }
    }

    fn enter(&self, _: &Id) {}

    fn exit(&self, _: &Id) {}
}

#[derive(Default)]
struct MessageVisitor {
    message: String,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn core::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        }
    }
}

#[test]
fn test_composite_polynomial_evaluation() {
    let a: Vec<TestScalar> = vec![
        -TestScalar::from(7u32),
        TestScalar::from(2u32),
        -TestScalar::from(6u32),
        TestScalar::from(17u32),
    ];
    let b: Vec<TestScalar> = vec![
        TestScalar::from(2u32),
        -TestScalar::from(8u32),
        TestScalar::from(4u32),
        TestScalar::from(1u32),
    ];
    let c: Vec<TestScalar> = vec![
        TestScalar::from(1u32),
        TestScalar::from(3u32),
        -TestScalar::from(5u32),
        -TestScalar::from(9u32),
    ];
    let mut prod = CompositePolynomial::new(2);
    prod.add_product([Rc::new(a), Rc::new(b)], TestScalar::from(3u32));
    prod.add_product([Rc::new(c)], TestScalar::from(2u32));
    let prod00 = prod.evaluate(&[TestScalar::from(0u32), TestScalar::from(0u32)]);
    let prod10 = prod.evaluate(&[TestScalar::from(1u32), TestScalar::from(0u32)]);
    let prod01 = prod.evaluate(&[TestScalar::from(0u32), TestScalar::from(1u32)]);
    let prod11 = prod.evaluate(&[TestScalar::from(1u32), TestScalar::from(1u32)]);
    let calc00 = -TestScalar::from(40u32);
    let calc10 = -TestScalar::from(42u32);
    let calc01 = -TestScalar::from(82u32);
    let calc11 = TestScalar::from(33u32);
    assert_eq!(prod00, calc00);
    assert_eq!(prod10, calc10);
    assert_eq!(prod01, calc01);
    assert_eq!(prod11, calc11);
}

#[expect(clippy::identity_op)]
#[test]
fn test_composite_polynomial_hypercube_sum() {
    let a: Vec<TestScalar> = vec![
        -TestScalar::from(7u32),
        TestScalar::from(2u32),
        -TestScalar::from(6u32),
        TestScalar::from(17u32),
    ];
    let b: Vec<TestScalar> = vec![
        TestScalar::from(2u32),
        -TestScalar::from(8u32),
        TestScalar::from(4u32),
        TestScalar::from(1u32),
    ];
    let c: Vec<TestScalar> = vec![
        TestScalar::from(1u32),
        TestScalar::from(3u32),
        -TestScalar::from(5u32),
        -TestScalar::from(9u32),
    ];
    let mut prod = CompositePolynomial::new(2);
    prod.add_product([Rc::new(a), Rc::new(b)], TestScalar::from(3u32));
    prod.add_product([Rc::new(c)], TestScalar::from(2u32));
    let sum = prod.hypercube_sum(4);
    assert_eq!(
        sum,
        TestScalar::from(3 * ((-7) * 2 + 2 * (-8) + (-6) * 4 + 17 * 1) + 2 * (1 + 3 + (-5) + (-9)))
    );
}

#[test]
fn test_composite_polynomial_annotate_trace_logs_each_product() {
    let a = Rc::new(vec![TestScalar::from(1u32), TestScalar::from(2u32)]);
    let b = Rc::new(vec![TestScalar::from(3u32), TestScalar::from(4u32)]);
    let c = Rc::new(vec![TestScalar::from(5u32), TestScalar::from(6u32)]);
    let mut prod = CompositePolynomial::new(1);
    prod.add_product([a, b], TestScalar::from(7u32));
    prod.add_product([c], TestScalar::from(11u32));

    let subscriber = ProductTraceSubscriber::default();
    let events = Arc::clone(&subscriber.events);
    tracing::subscriber::with_default(subscriber, || prod.annotate_trace());

    let events = events.lock().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(
        events[0],
        format!("Product #0: {:#} * [0, 1]", TestScalar::from(7u32))
    );
    assert_eq!(
        events[1],
        format!("Product #1: {:#} * [2]", TestScalar::from(11u32))
    );
}
