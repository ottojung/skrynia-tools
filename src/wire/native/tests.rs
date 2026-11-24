use super::*;
#[test]
fn native_frames_and_conflicts() {
    let cap = [9; 32];
    let mut prefix = vec![1, 7, 32, 0];
    prefix.extend(cap);
    prefix.extend([1, 0, 0, 0, b'a']);
    assert_eq!(get(&cap, "a").unwrap(), prefix);
    prefix[1] = 6;
    prefix.extend([0, 1, 0, 0, 0, b'x']);
    assert_eq!(put(&cap, "a", None, b"x").unwrap(), prefix);
    assert!(get(&cap, ".bad").is_err());
    assert_eq!(read(&[1, 1, 1, 0, 0, 0, 8]).unwrap(), None);
    assert!(!written(&[1, 1, 2, 0, 0, 0, 10, 0], b"x").unwrap());
    assert!(written(&[1, 1, 1, 0, 0, 0, 10], b"x").is_err());
    assert!(written(&[1, 0, 0, 0, 0, 0], b"x").is_err());
    let mut reply = vec![1, 0, 32, 0, 0, 0];
    reply.extend(digest(b"x"));
    assert!(written(&reply, b"x").unwrap());
    assert!(written(&reply, b"y").is_err());
    reply.push(0);
    assert!(written(&reply, b"x").is_err());
}
