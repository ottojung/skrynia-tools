use std::cell::RefCell;

type Reply = Result<(i32, Vec<u8>), i32>;

thread_local! {
    static LOG: RefCell<Vec<Vec<u8>>> = RefCell::new(Vec::new());
    static FAIL_SYNC: RefCell<Option<usize>> = RefCell::new(None);
    static SYNC_COUNT: RefCell<usize> = RefCell::new(0);
    static NEXT_DIR_FD: RefCell<i32> = RefCell::new(100);
}

/// `fail_sync` is the 1-based SYNC ordinal that replies with -5.
pub fn configure(fail_sync: Option<usize>) {
    LOG.with(|l| l.borrow_mut().clear());
    FAIL_SYNC.with(|f| *f.borrow_mut() = fail_sync);
    SYNC_COUNT.with(|c| *c.borrow_mut() = 0);
    NEXT_DIR_FD.with(|n| *n.borrow_mut() = 100);
}

pub fn take_log() -> Vec<Vec<u8>> {
    LOG.with(|l| std::mem::take(&mut *l.borrow_mut()))
}

pub(super) fn sync_reply() -> Reply {
    let ordinal = SYNC_COUNT.with(|c| {
        let mut c = c.borrow_mut();
        *c += 1;
        *c
    });
    let fail = FAIL_SYNC.with(|f| *f.borrow());
    Ok((if fail == Some(ordinal) { -5 } else { 0 }, Vec::new()))
}

pub(super) fn dir_fd() -> i32 {
    NEXT_DIR_FD.with(|n| {
        let mut n = n.borrow_mut();
        let fd = *n;
        *n += 1;
        fd
    })
}

pub fn call(req: &[u8]) -> Reply {
    LOG.with(|l| l.borrow_mut().push(req.to_vec()));
    super::dispatch::reply(req)
}
