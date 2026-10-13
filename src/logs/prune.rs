//! Prunes the log store after every write. This is a stub until S1.3 gives it the keep_logs rule of design §4.3, so it deletes nothing yet.
//! Cases: none (the stub deletes nothing; LOG-PRUNE-* are planned for S1.3)

use std::path::Path;

use crate::clock::Timestamp;
use crate::error::Error;

/// Prunes `store` by `keep_logs` as of `now`. Until S1.3 it changes nothing and cannot fail; the signature is final.
pub fn run(store: &Path, keep_logs: &str, now: Timestamp) -> Result<(), Error> {
    let _ = (store, keep_logs, now);
    Ok(())
}
