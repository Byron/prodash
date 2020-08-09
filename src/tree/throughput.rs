use crate::{progress, tree, unit};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
struct State {
    desired: std::time::Duration,
    observed: std::time::Duration,
    aggregate_value_for_observed_duration: progress::Step,
    last_value: progress::Step,
}

impl State {
    fn new(value: progress::Step) -> Self {
        State {
            desired: std::time::Duration::from_secs(1),
            observed: Default::default(),
            aggregate_value_for_observed_duration: 0,
            last_value: value,
        }
    }
}

pub struct Throughput {
    sorted_by_key: Vec<(tree::Key, State)>,
}

impl Throughput {
    pub fn update_and_get(&mut self, key: &tree::Key, value: &progress::Value) -> Option<unit::display::Throughput> {
        unimplemented!("update and get")
    }
    pub fn reconcile(&mut self, values: &[(tree::Key, progress::Value)]) {
        unimplemented!("reconcile")
    }
}
