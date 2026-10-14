//! Maps a function over a list on several threads, and keeps the results in the list's order.
//! Cases: none (an internal helper: the results are those of a sequential map, in the same order)

use std::num::NonZeroUsize;
use std::panic;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

/// The most threads that a map or a walk runs on.
const MAX_THREADS: usize = 8;

/// How many items a thread takes from the shared count at a time.
const BATCH: usize = 64;

/// The threads that a parallel step may use: the machine's parallelism, at most `MAX_THREADS`, and one when it is unknown.
pub fn threads() -> usize {
    thread::available_parallelism()
        .map_or(1, NonZeroUsize::get)
        .min(MAX_THREADS)
}

/// `work` applied to every item, spread over `threads()` threads in batches of at most `BATCH` items, with the results in the
/// order of `items`. A panic in `work` is raised again on the calling thread.
pub fn map<T, R, F>(items: &[T], work: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Sync,
{
    let workers = threads().min(items.len());
    if workers <= 1 {
        return items.iter().map(work).collect();
    }
    // Four batches a thread at most, so that a thread that runs out of work does not wait long for the last one.
    let batch = items.len().div_ceil(workers * 4).clamp(1, BATCH);
    let next = AtomicUsize::new(0);
    let mut done: Vec<(usize, R)> = thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| scope.spawn(|| run_batches(items, batch, &next, &work)))
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|panic| panic::resume_unwind(panic))
            })
            .collect()
    });
    done.sort_by_key(|(index, _)| *index);
    done.into_iter().map(|(_, result)| result).collect()
}

/// Takes batches of `batch` indexes from `next` until none are left, and returns each one's result with its index.
fn run_batches<T, R, F>(items: &[T], batch: usize, next: &AtomicUsize, work: &F) -> Vec<(usize, R)>
where
    F: Fn(&T) -> R,
{
    let mut done = Vec::new();
    loop {
        let start = next.fetch_add(batch, Ordering::Relaxed);
        if start >= items.len() {
            return done;
        }
        let end = items.len().min(start + batch);
        done.extend((start..end).map(|index| (index, work(&items[index]))));
    }
}

#[cfg(test)]
mod tests {
    use super::map;

    #[test]
    fn the_results_come_back_in_the_order_of_the_items() {
        let items: Vec<usize> = (0..1_000).collect();
        let doubled = map(&items, |item| item * 2);
        assert_eq!(
            doubled,
            items.iter().map(|item| item * 2).collect::<Vec<_>>()
        );
    }

    #[test]
    fn an_empty_list_maps_to_an_empty_list() {
        let none: Vec<usize> = Vec::new();
        assert!(map(&none, |item| *item).is_empty());
    }
}
