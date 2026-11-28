use super::replies;

type Reply = Result<(i32, Vec<u8>), i32>;

pub fn configure(fail_sync: Option<usize>) {
    replies::configure(fail_sync)
}

pub fn take_log() -> Vec<Vec<u8>> {
    replies::take_log()
}

pub fn call(req: &[u8]) -> Reply {
    replies::call(req)
}

pub fn call_with_capacity(req: &[u8], _cap: usize) -> Reply {
    call(req)
}
