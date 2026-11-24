use std::cell::RefCell;

const MAX: usize = 8;

thread_local! {
    // Fresh per root invocation: the kernel instantiates Program 0 per request.
    static PENDING: RefCell<Vec<(bool, Vec<u8>)>> = const { RefCell::new(Vec::new()) };
}

/// A mediated child only queues a request. Execution happens after evaluation,
/// when the kernel restores the root's private operator-disk authority.
pub fn enqueue(publish: bool, path: &[u8]) -> bool {
    if !super::path::is_public(path) {
        return false;
    }
    PENDING.with(|queue| {
        let mut queue = queue.borrow_mut();
        if queue.len() >= MAX {
            return false;
        }
        queue.push((publish, path.to_vec()));
        true
    })
}

pub fn drain() -> Vec<(bool, Vec<u8>)> {
    PENDING.with(|queue| core::mem::take(&mut *queue.borrow_mut()))
}
