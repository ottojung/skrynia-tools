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

#[test]
fn administrative_revocations_require_exact_confirmation_and_persist_redacted_state() {
    support::provision::revoke::verify();
}
#[test]
fn uncertain_revocation_requires_ack_and_accepts_confirmed_absence() {
    support::provision::revoke::retry();
}

#[test]
fn rejected_and_malformed_revocation_responses_do_not_erase_capabilities() {
    support::provision::revoke::failures();
}
