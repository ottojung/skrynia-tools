//! Raw HTTP/1.1 responses for the disposable mock edge.
use super::Reply;

pub fn status(auth: bool, route: bool) -> u16 {
    if !auth {
        401
    } else if !route {
        403
    } else {
        0
    }
}

pub fn respond(reply: &Reply, failure: u16) -> Vec<u8> {
    if matches!(reply, Reply::Hang) {
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    let (code, body) = outcome(reply, failure);
    let location = match reply {
        Reply::Redirect(port) => format!("Location: http://127.0.0.1:{port}/_skrynia/\r\n"),
        _ => String::new(),
    };
    let head = format!(
        "HTTP/1.1 {code} X\r\nContent-Length: {}\r\n{location}Connection: close\r\n\r\n",
        body.len()
    );
    head.into_bytes().into_iter().chain(body).collect()
}

fn outcome(reply: &Reply, failure: u16) -> (u16, Vec<u8>) {
    if failure != 0 {
        return (failure, Vec::new());
    }
    match reply {
        Reply::Ok(body) => (200, body.clone()),
        Reply::Status(code) => (*code, Vec::new()),
        Reply::Oversized => (200, vec![7u8; 9000]),
        Reply::Hang => (200, b"late".to_vec()),
        Reply::Redirect(_) => (302, Vec::new()),
    }
}
