use crate::wire::{
    OP_APP_GET, OP_APP_PUT, OP_INVOKE, OP_LOOKUP, OP_REGISTER, OP_REVOKE, OP_SET, Request, VERSION,
};

pub fn encode_req(req: &Request) -> Vec<u8> {
    let mut out = vec![VERSION];
    match req {
        Request::Register { code } => {
            out.push(OP_REGISTER);
            put_blob(&mut out, code);
        }
        Request::Set { id, code } => {
            out.push(OP_SET);
            put_id(&mut out, id);
            put_blob(&mut out, code);
        }
        Request::Invoke { id, input } => {
            out.push(OP_INVOKE);
            put_id(&mut out, id);
            put_blob(&mut out, input);
        }
        Request::Lookup { id } => {
            out.push(OP_LOOKUP);
            put_id(&mut out, id);
        }
        Request::Revoke { id } => {
            out.push(OP_REVOKE);
            put_id(&mut out, id);
        }
        Request::AppPut { app, key, value } => {
            out.push(OP_APP_PUT);
            put_id(&mut out, app);
            put_blob(&mut out, key);
            put_blob(&mut out, value);
        }
        Request::AppGet { app, key } => {
            out.push(OP_APP_GET);
            put_id(&mut out, app);
            put_blob(&mut out, key);
        }
    }
    out
}

fn put_id(out: &mut Vec<u8>, id: &[u8]) {
    out.extend_from_slice(&(id.len() as u16).to_le_bytes());
    out.extend_from_slice(id);
}

fn put_blob(out: &mut Vec<u8>, data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(data);
}
