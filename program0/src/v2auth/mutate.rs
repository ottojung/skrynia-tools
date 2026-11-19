#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutateKind {
    Create,
    Replace,
    Delete,
}

#[derive(Debug, Clone)]
pub struct MutateReq<'a> {
    pub token: [u8; 32],
    pub key: &'a str,
    pub kind: MutateKind,
    pub expected: Option<[u8; 32]>,
    pub body: &'a [u8],
}

#[derive(Debug)]
pub enum MutateErr<E> {
    Disk(E),
    Decode,
    Uncertain,
    MissingAuthority,
    AuthorityExists,
    InvalidToken,
    Forbidden,
    NotFound,
    Exists,
    ContentMismatch,
    BadKey,
}

impl<E> MutateErr<E> {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Uncertain | Self::Disk(_) | Self::MissingAuthority | Self::Decode => {
                "state_uncertain"
            }
            Self::InvalidToken => "invalid_grant_user_token",
            Self::Forbidden => "grant_operation_forbidden",
            Self::NotFound => "not_found",
            Self::Exists => "already_exists",
            Self::ContentMismatch => "content_mismatch",
            Self::AuthorityExists => "invalid_state",
            Self::BadKey => "invalid_key",
        }
    }
}
