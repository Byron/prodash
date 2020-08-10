use crate::{progress, unit};
use std::collections::VecDeque;
use std::time::{Duration, SystemTime};

const THROTTLE_INTERVAL: Duration = Duration::from_secs(1);
const ONCE_A_SECOND: Duration = Duration::from_secs(1);

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
struct State {
    observed: Duration,
    last_value: progress::Step,
    elapsed_values: VecDeque<(Duration, progress::Step)>,

    last_update_duration: Duration,
    precomputed_throughput: Option<progress::Step>,
}

impl State {
    fn new(value: progress::Step, elapsed: Duration) -> Self {
        State {
            observed: elapsed,
            last_value: value,
            elapsed_values: {
                let mut v = VecDeque::with_capacity(6); // default frames per second
                v.push_back((elapsed, value));
                v
            },

            last_update_duration: elapsed,
            precomputed_throughput: None,
        }
    }

    fn compute_throughput(&mut self) -> progress::Step {
        let mut observed: Duration = self.elapsed_values.iter().map(|e| e.0).sum();
        while !self.elapsed_values.is_empty() && observed > ONCE_A_SECOND {
            let candidate = self
                .elapsed_values
                .front()
                .map(|e| e.0)
                .expect("at least one item as we are in the checked loop");
            if observed.checked_sub(candidate).unwrap_or_default() <= ONCE_A_SECOND {
                break;
            }
            observed -= candidate;
            self.elapsed_values.pop_front();
        }
        let observed_value: progress::Step = self.elapsed_values.iter().map(|e| e.1).sum();
        ((observed_value as f64 / observed.as_secs_f64()) * ONCE_A_SECOND.as_secs_f64()) as progress::Step
    }

    fn update(&mut self, value: progress::Step, elapsed: Duration) -> Option<unit::display::Throughput> {
        self.observed += elapsed;
        self.elapsed_values
            .push_back((elapsed, value.saturating_sub(self.last_value)));
        self.last_value = value;
        if self.observed - self.last_update_duration > THROTTLE_INTERVAL {
            self.precomputed_throughput = Some(self.compute_throughput());
            self.last_update_duration = self.observed;
        }
        self.throughput()
    }

    fn throughput(&self) -> Option<unit::display::Throughput> {
        self.precomputed_throughput.map(|tp| unit::display::Throughput {
            value_change_in_timespan: tp,
            timespan: ONCE_A_SECOND,
        })
    }
}

#[derive(Default)]
pub struct Throughput {
    sorted_by_key: Vec<(progress::Key, State)>,
    updated_at: Option<SystemTime>,
    elapsed: Option<Duration>,
}

impl Throughput {
    pub fn update_elapsed(&mut self) {
        let now = SystemTime::now();
        self.elapsed = self.updated_at.and_then(|then| now.duration_since(then).ok());
        self.updated_at = Some(now);
    }

    pub fn update_and_get(
        &mut self,
        key: &progress::Key,
        progress: Option<&progress::Value>,
    ) -> Option<unit::display::Throughput> {
        progress.and_then(|progress| {
            self.elapsed
                .and_then(|elapsed| match self.sorted_by_key.binary_search_by_key(key, |t| t.0) {
                    Ok(index) => self.sorted_by_key[index].1.update(progress.step, elapsed),
                    Err(index) => {
                        let state = State::new(progress.step, elapsed);
                        let tp = state.throughput();
                        self.sorted_by_key.insert(index, (*key, state));
                        tp
                    }
                })
        })
    }
    pub fn reconcile(&mut self, sorted_values: &[(progress::Key, progress::Task)]) {
        self.sorted_by_key
            .retain(|(key, _)| sorted_values.binary_search_by_key(key, |e| e.0).is_ok());
    }
}
