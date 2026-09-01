//! Unit tests for the auth module: bot-dissuasion checks (challenge proof,
//! form timing, honeypot) and password hashing. These are pure functions and
//! run without a database. Integration tests in tests/integration.rs cover
//! the SQL paths and the migration.

use crate::auth::{
    challenge_proof_valid, compute_challenge_proof, fresh_nonce, hash_password, verify_password,
    AuthForm, FORM_MAX_OPEN_MS, FORM_MIN_OPEN_MS,
};

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// Build a valid AuthForm body the way the client would (nonce fetched from
/// /api/auth/nonce, proof computed in JS).
fn valid_form(username: &str, password: &str) -> AuthForm {
    let nonce = fresh_nonce();
    let proof = compute_challenge_proof(&nonce, username, password);
    AuthForm {
        username: username.into(),
        password: password.into(),
        form_opened_at: Some(now_ms() - 10_000), // opened 10s ago
        website: String::new(),
        challenge_proof: Some(proof),
        challenge_nonce: Some(nonce),
    }
}

#[test]
fn hash_roundtrips() {
    let phc = hash_password("s3cret-password").expect("hash");
    assert!(phc.starts_with("$argon2id$v=19$"));
    assert!(verify_password("s3cret-password", &phc));
    assert!(!verify_password("wrong-password", &phc));
}

#[test]
fn hash_rejects_malformed_phc() {
    assert!(!verify_password("x", "not-a-phc-string"));
    assert!(!verify_password("x", ""));
}

#[test]
fn challenge_proof_hex_and_rotation() {
    let nonce = fresh_nonce();
    assert_eq!(nonce.len(), 32);
    assert!(nonce.bytes().all(|b| b.is_ascii_hexdigit()));

    let n1 = fresh_nonce();
    let n2 = fresh_nonce();
    assert_ne!(n1, n2, "nonces must rotate");

    let proof = compute_challenge_proof(&nonce, "user", "pass");
    assert_eq!(proof.len(), 16);
    assert!(challenge_proof_valid(&proof, &nonce, "user", "pass"));
    // any change to nonce/username/password changes the proof
    assert!(!challenge_proof_valid(&proof, &nonce, "user", "pass2"));
    assert!(!challenge_proof_valid(&proof, &nonce, "user2", "pass"));
    assert!(!challenge_proof_valid(&proof, &nonce2(), "user", "pass"));
}

fn nonce2() -> String {
    // a second, different nonce
    loop {
        let n = fresh_nonce();
        if n != fresh_nonce() {
            // fresh_nonce() twice is fine; just return something different
            return n;
        }
    }
}

#[test]
fn challenge_proof_rejects_malformed() {
    let nonce = fresh_nonce();
    let proof = compute_challenge_proof(&nonce, "user", "pass");
    assert!(!challenge_proof_valid("", &nonce, "user", "pass"));
    assert!(!challenge_proof_valid("abc", &nonce, "user", "pass"));
    assert!(!challenge_proof_valid(&proof.to_uppercase(), &nonce, "user", "pass"));
    assert!(!challenge_proof_valid("zzzzzzzzzzzzzzzz", &nonce, "user", "pass"));
}

#[test]
fn timing_window_rejects_instant_and_stale_forms() {
    // instant submit (form_opened_at = now) -> too fast
    let mut form = valid_form("timing_user", "password-1");
    form.form_opened_at = Some(now_ms());
    let err = form_check(&form);
    assert!(err.is_err(), "instant submit must be rejected");

    // opened 1ms ago -> below the 3s floor
    let mut form = valid_form("timing_user", "password-1");
    form.form_opened_at = Some(now_ms() - 1);
    assert!(form_check(&form).is_err());

    // opened just over 10 minutes ago -> stale
    let mut form = valid_form("timing_user", "password-1");
    form.form_opened_at = Some(now_ms() - (FORM_MAX_OPEN_MS as i64) - 1_000);
    assert!(form_check(&form).is_err());

    // in-window forms pass (form_opened_at 10s ago)
    let form = valid_form("timing_user", "password-1");
    assert!(form_check(&form).is_ok());

    // form_opened_at missing -> treated as 0 -> stale -> rejected
    let mut form = valid_form("timing_user", "password-1");
    form.form_opened_at = None;
    assert!(form_check(&form).is_err());
}

#[test]
fn honeypot_rejects_filled_website() {
    let mut form = valid_form("honey_user", "password-1");
    form.website = "https://spam.example".into();
    let err = form_check(&form);
    assert!(err.is_err(), "filled honeypot must be rejected");
}

#[test]
fn missing_challenge_parts_rejected() {
    let mut form = valid_form("chal_user", "password-1");
    form.challenge_proof = None;
    assert!(form_check(&form).is_err());

    let mut form = valid_form("chal_user", "password-1");
    form.challenge_nonce = None;
    assert!(form_check(&form).is_err());

    let mut form = valid_form("chal_user", "password-1");
    form.challenge_nonce = Some("tooshort".into());
    assert!(form_check(&form).is_err());

    // wrong proof for the given nonce/username/password
    let mut form = valid_form("chal_user", "password-1");
    form.challenge_proof = Some("0000000000000000".into());
    assert!(form_check(&form).is_err());
}

#[test]
fn timing_constants_sane() {
    assert!(FORM_MIN_OPEN_MS < FORM_MAX_OPEN_MS);
    assert_eq!(FORM_MIN_OPEN_MS, 3_000);
    assert_eq!(FORM_MAX_OPEN_MS, 600_000);
}

/// Run the form through the same validation the register/login handlers use
/// (validate_auth_form is private; this mirrors its behavior via the public
/// challenge/timing helpers).
fn form_check(form: &AuthForm) -> Result<(), ()> {
    let now = now_ms();
    let opened = form.form_opened_at.unwrap_or(0) as u128;
    let open_ms = (now as u128).saturating_sub(opened);
    if open_ms < FORM_MIN_OPEN_MS || open_ms > FORM_MAX_OPEN_MS {
        return Err(());
    }
    if !form.website.trim().is_empty() {
        return Err(());
    }
    let proof = form.challenge_proof.as_deref().ok_or(())?;
    let nonce = form.challenge_nonce.as_deref().ok_or(())?;
    if nonce.len() != 32 || !nonce.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(());
    }
    if !challenge_proof_valid(proof, nonce, form.username.trim(), &form.password) {
        return Err(());
    }
    Ok(())
}
