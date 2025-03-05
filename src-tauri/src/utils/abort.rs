use std::sync::atomic::{AtomicBool, Ordering};
use lazy_static::lazy_static;

lazy_static! {
    static ref ABORT_FLAG: AtomicBool = AtomicBool::new(false);
}

/// Check if synchronization should be aborted
pub fn should_abort() -> bool {
    ABORT_FLAG.load(Ordering::SeqCst)
}

/// Reset the abort flag
pub fn reset_abort_flag() {
    ABORT_FLAG.store(false, Ordering::SeqCst);
}

/// Set the abort flag to stop synchronization
pub fn set_abort_flag() {
    ABORT_FLAG.store(true, Ordering::SeqCst);
}