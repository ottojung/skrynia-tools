use super::{call, sync_dir, wire};

pub fn mkdir_p(dir: &str) -> Result<(), i32> {
    let mut cur = String::new();
    let mut root = true;
    for part in dir.trim_end_matches('/').split('/') {
        if part.is_empty() {
            continue;
        }
        let parent = cur.clone();
        if !cur.is_empty() {
            cur.push('/');
        }
        cur.push_str(part);
        // The leading root token selects the namespace; it is not a
        // directory to create.
        if root && (cur == "user" || cur == "operator") {
            root = false;
            continue;
        }
        root = false;
        let mut r = vec![wire::MKDIR];
        wire::put_bytes(&mut r, cur.as_bytes());
        // create + non-exclusive: existing directories are no-op success.
        wire::put_u32(&mut r, 4);
        let (n, _) = call::call(&r)?;
        if n < 0 {
            return Err(n);
        }
        sync_parent(&parent)?;
    }
    Ok(())
}

fn sync_parent(parent: &str) -> Result<(), i32> {
    // mkdir is not durable by itself. Sync the parent unconditionally:
    // existing-path success is not proof of durability, so a retry after a
    // failed sync cannot skip any ancestor.
    if parent == "user" || parent == "operator" {
        sync_dir(&format!("{parent}/."))
    } else {
        sync_dir(parent)
    }
}
