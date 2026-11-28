pub use super::cap::{call, disk_atom, wire};

pub fn path(root: &str) -> String {
    format!("{root}/a/b/c/record.bin")
}

pub fn dirs(root: &str) -> Vec<String> {
    vec![
        format!("{root}/."),
        format!("{root}/a"),
        format!("{root}/a/b"),
        format!("{root}/a/b/c"),
    ]
}

pub fn ops(log: &[Vec<u8>]) -> Vec<u8> {
    log.iter().map(|req| req[0]).collect()
}

pub fn first_path(req: &[u8]) -> String {
    let n = u32::from_le_bytes(req[1..5].try_into().unwrap()) as usize;
    String::from_utf8(req[5..5 + n].to_vec()).unwrap()
}

pub fn fd(req: &[u8]) -> i32 {
    i32::from_le_bytes(req[1..5].try_into().unwrap())
}

pub fn opened_dirs(log: &[Vec<u8>]) -> Vec<String> {
    log.iter()
        .filter(|req| req[0] == wire::OPEN_DIR)
        .map(|req| first_path(req))
        .collect()
}

pub fn sync_fds(log: &[Vec<u8>]) -> Vec<i32> {
    log.iter()
        .filter(|req| req[0] == wire::SYNC)
        .map(|req| fd(req))
        .collect()
}
