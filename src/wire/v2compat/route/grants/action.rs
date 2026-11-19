pub enum Action<'a> {
    Create,
    Inspect(&'a str),
    Issue(&'a str),
    RevokeUser(&'a str, &'a str),
    Revoke(&'a str),
}
impl<'a> Action<'a> {
    pub fn parse(method: &str, path: &'a str) -> Option<Self> {
        let parts: Vec<&str> = path.split('/').collect();
        match (parts.as_slice(), method) {
            (["", "grants"], "POST") => Some(Self::Create),
            (["", "grants", id], "GET") if !id.is_empty() => Some(Self::Inspect(id)),
            (["", "grants", id, "users"], "POST") if !id.is_empty() => Some(Self::Issue(id)),
            (["", "grants", id, "users", user], "DELETE") if !id.is_empty() && !user.is_empty() => {
                Some(Self::RevokeUser(id, user))
            }
            (["", "grants", id], "DELETE") if !id.is_empty() => Some(Self::Revoke(id)),
            _ => None,
        }
    }
    pub fn issuer_only(&self) -> bool {
        matches!(
            self,
            Self::Issue(_) | Self::RevokeUser(_, _) | Self::Revoke(_)
        )
    }
}
