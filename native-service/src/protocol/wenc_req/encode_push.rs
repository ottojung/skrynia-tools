use crate::wire::Request;
pub fn is_push(req: &Request) -> bool {
    matches!(
        req,
        Request::PushSubscribe(..)
            | Request::PushUnsubscribe(..)
            | Request::PushGrant(..)
            | Request::PushSenderGrant(..)
            | Request::PushWakeGrant(..)
            | Request::SubStatus(..)
    )
}
pub fn encode_push(out: &mut Vec<u8>, req: &Request) {
    match req {
        Request::PushSubscribe(g, k, s, sub) => {
            super::encode_push_ops::subscribe(out, g, k, s, sub)
        }
        Request::PushUnsubscribe(g, k, s) => super::encode_push_ops::unsubscribe(out, g, k, s),
        Request::PushGrant(w, g, k, s) => super::encode_push_ops::grant(out, w, g, k, s),
        Request::PushSenderGrant(w, g, k) => super::encode_push_ops::sender_grant(out, w, g, k),
        Request::PushWakeGrant(w, g, k, ns) => super::encode_push_ops::wake_grant(out, w, g, k, ns),
        Request::SubStatus(g, k, s, h) => super::encode_push_ops::sub_status(out, g, k, s, h),
        _ => unreachable!(),
    }
}
