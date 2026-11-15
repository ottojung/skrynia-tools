mod support;
#[test]
fn pending_creation_is_durable_and_inspectable() {
    support::provision::pending::verify();
}
#[test]
fn resumable_issuer_credentials_can_issue_user_token() {
    support::provision::resume::verify();
}
#[test]
fn uncertain_issuance_requires_explicit_duplicate_risk_ack() {
    support::provision::uncertain::verify();
}

#[test]
fn lost_user_token_reply_preserves_issuer_and_requires_manual_recovery() {
    support::provision::create::verify();
}
