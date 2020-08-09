use crate::{progress, tree, unit};
use std::time::Duration;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
struct State {
    desired: Duration,
    observed: Duration,
    aggregate_value_for_observed_duration: progress::Step,
    last_value: progress::Step,
}

impl State {
    fn new(value: progress::Step, elapsed: Duration) -> Self {
        State {
            desired: Duration::from_secs(1),
            observed: elapsed,
            aggregate_value_for_observed_duration: value,
            last_value: value,
        }
    }
    fn update(&mut self, value: progress::Step, elapsed: Duration) -> Option<unit::display::Throughput> {
        self.throughput()
    }

    fn throughput(&self) -> Option<unit::display::Throughput> {
        Some(unit::display::Throughput {
            value_change_in_timespan: self.aggregate_value_for_observed_duration,
            timespan: self.desired,
        })
    }
}

#[derive(Default)]
pub struct Throughput {
    sorted_by_key: Vec<(tree::Key, State)>,
}

impl Throughput {
    pub fn update_and_get(
        &mut self,
        key: &tree::Key,
        value: &progress::Value,
        elapsed: Duration,
    ) -> Option<unit::display::Throughput> {
        value
            .progress
            .as_ref()
            .and_then(|progress| match self.sorted_by_key.binary_search_by_key(key, |t| t.0) {
                Ok(index) => self.sorted_by_key[index].1.update(progress.step, elapsed),
                Err(index) => {
                    let state = State::new(progress.step, elapsed);
                    let tp = state.throughput();
                    self.sorted_by_key.insert(index, (*key, state));
                    tp
                }
            })
    }
    pub fn reconcile(&mut self, sorted_values: &[(tree::Key, progress::Value)]) {
        self.sorted_by_key
            .retain(|(key, _)| sorted_values.binary_search_by_key(key, |e| e.0).is_ok());
    }
}
