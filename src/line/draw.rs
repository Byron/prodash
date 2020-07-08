use crate::tree;
use std::{io, ops::RangeInclusive};

#[derive(Default)]
pub struct State {
    tree: Vec<(tree::Key, tree::Value)>,
}

pub struct Options {
    pub level_filter: Option<RangeInclusive<tree::Level>>,
    pub keep_running_if_progress_is_empty: bool,
}

pub fn lines(_out: &mut impl io::Write, progress: &tree::Root, state: &mut State, config: &Options) -> io::Result<()> {
    progress.sorted_snapshot(&mut state.tree);
    if !config.keep_running_if_progress_is_empty && state.tree.is_empty() {
        return Err(io::Error::new(io::ErrorKind::Other, "stop as progress is empty"));
    }
    // progress.copy_new_messages()
    let level_range = config
        .level_filter
        .clone()
        .unwrap_or(RangeInclusive::new(0, tree::Level::max_value()));
    for (_key, _progress) in state.tree.iter().filter(|(k, _)| level_range.contains(&k.level())) {
        unimplemented!("drawing to be done")
    }
    Ok(())
}
