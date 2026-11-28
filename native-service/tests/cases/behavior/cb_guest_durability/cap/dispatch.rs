use super::replies;

type Reply = Result<(i32, Vec<u8>), i32>;

pub fn reply(req: &[u8]) -> Reply {
    let op = req[0];
    if op == super::wire::SYNC {
        return replies::sync_reply();
    }
    match op {
        super::wire::MKDIR | super::wire::CLOSE | super::wire::RENAME | super::wire::DELETE => {
            Ok((0, Vec::new()))
        }
        super::wire::OPEN_DIR => Ok((replies::dir_fd(), Vec::new())),
        super::wire::RANDOM => Ok((16, vec![0x5a; 16])),
        super::wire::OPEN_DISK if req.windows(7).any(|w| w == b"missing") => {
            Ok((super::wire::E_NOENT, Vec::new()))
        }
        super::wire::OPEN_DISK => Ok((1, Vec::new())),
        super::wire::WRITE => {
            let n = u32::from_le_bytes(req[5..9].try_into().unwrap()) as i32;
            Ok((n, Vec::new()))
        }
        _ => Err(-1),
    }
}
