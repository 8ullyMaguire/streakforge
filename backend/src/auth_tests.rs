//! Unit tests for the auth module: bot-dissuasion checks (challenge proof,
//! form timing, honeypot) and password hashing. These are pure functions and
//! run without a database. Integration tests in tests/integration.rs cover
//! the SQL paths and the migration.

use crate::auth::{
    challenge_proof_valid, client_ip, compute_challenge_proof, fresh_nonce, hash_password,
    escalating_lockout_secs, session_cookie_name, throttle_verdict, verify_password, AuthForm,
    LOGIN_ATTEMPT_KEEP_SECS, LOGIN_ESCALATE_AFTER, LOGIN_MAX_LOCKOUT_SECS, LOGIN_MAX_PER_IP,
    LOGIN_MAX_PER_USERNAME,
    LOGIN_WINDOW_SECS, FORM_MAX_OPEN_MS, FORM_MIN_OPEN_MS,
};

// ---- Session cookie naming -------------------------------------------------

#[test]
fn host_prefix_applies_only_when_the_cookie_is_secure() {
    // `__Host-` without `Secure` is rejected by every browser, so sending the
    // prefixed name over local HTTP would mean no session at all in dev.
    assert_eq!(session_cookie_name(false), "streakforge_session");
    assert_eq!(session_cookie_name(true), "__Host-streakforge_session");
    // The prefix is a browser guarantee only when the name starts with it
    // exactly -- "__host-" (lowercase) is just a name.
    assert!(session_cookie_name(true).starts_with("__Host-"));
}

// ---- Failed-login throttle -------------------------------------------------

#[test]
fn throttle_allows_below_the_ceiling_and_blocks_at_it() {
    // Last allowed attempt: one under the limit.
    assert_eq!(throttle_verdict(LOGIN_MAX_PER_USERNAME - 1, 0, Some(120), 0), None);
    // The attempt that trips it.
    assert!(throttle_verdict(LOGIN_MAX_PER_USERNAME, 0, Some(120), 0).is_some());
    // The IP ceiling is independent and higher: five failures — enough to trip
    // one username — must NOT block the address, or a shared household or cafe
    // would lock out everyone at the first guesser's fifth try.
    assert_eq!(throttle_verdict(0, LOGIN_MAX_PER_USERNAME, Some(120), 0), None);
    assert!(throttle_verdict(0, LOGIN_MAX_PER_IP, Some(120), 0).is_some());
    assert!(LOGIN_MAX_PER_IP > LOGIN_MAX_PER_USERNAME);
}

#[test]
fn throttle_never_reports_zero_seconds() {
    // `Retry-After: 0` invites an immediate retry that is refused again, which
    // reads as a broken client contract rather than as a wait.
    let blocked = throttle_verdict(99, 0, None, 0).expect("over the ceiling is blocked");
    assert!(blocked >= 1, "a blocked verdict must report at least one second");
    let blocked = throttle_verdict(99, 0, Some(0), 0).expect("still blocked");
    assert_eq!(blocked, 1);
    // With no row left to time the window from, the full window is the answer.
    let blocked = throttle_verdict(99, 0, None, 0).unwrap();
    assert_eq!(blocked, LOGIN_WINDOW_SECS);
}

#[test]
fn the_lockout_doubles_past_the_days_allowance_of_typos() {
    // Up to the day's allowance: no escalation at all. Five is what a person
    // reaches by mistyping, so the sixth failure is not treated as hostile --
    // the base window already answers it.
    for n in 0..=LOGIN_ESCALATE_AFTER {
        assert_eq!(escalating_lockout_secs(n), None, "{n} failures is still typo territory");
    }
    // Past it: double the base window each time, then hold at the ceiling.
    assert_eq!(escalating_lockout_secs(9), Some(LOGIN_WINDOW_SECS * 2));
    assert_eq!(escalating_lockout_secs(10), Some(LOGIN_WINDOW_SECS * 4));
    assert_eq!(escalating_lockout_secs(11), Some(LOGIN_MAX_LOCKOUT_SECS));
    // A cap is the point: this password has no self-service reset, so a lockout
    // with no way back would be a worse failure than the guesses it prevents.
    assert_eq!(escalating_lockout_secs(40), Some(LOGIN_MAX_LOCKOUT_SECS), "the wait is capped");
    assert!(LOGIN_MAX_LOCKOUT_SECS <= LOGIN_ATTEMPT_KEEP_SECS);
}

#[test]
fn the_window_is_the_trigger_and_the_day_is_the_multiplier() {
    // Under the window, a persistent day does NOT block on its own: the
    // escalation lengthens a refusal, it never invents one.
    assert_eq!(throttle_verdict(1, 0, None, 40), None);
    // Over the window on a clean day: the sliding window answers.
    assert_eq!(throttle_verdict(LOGIN_MAX_PER_USERNAME, 0, Some(120), 5), Some(120));
    // Over the window after a persistent day: the escalation answers instead,
    // and it is the longer of the two numbers.
    assert_eq!(throttle_verdict(LOGIN_MAX_PER_USERNAME, 0, Some(120), 11), Some(LOGIN_MAX_LOCKOUT_SECS));
    assert_eq!(throttle_verdict(LOGIN_MAX_PER_USERNAME, 0, Some(0), 9), Some(LOGIN_WINDOW_SECS * 2));
    // An address stays on the plain window: a cafe shares it, so escalating it
    // would punish the neighbours for one attacker. Only the username escalates.
    assert_eq!(throttle_verdict(0, LOGIN_MAX_PER_IP, Some(120), 40), Some(120));
}

#[test]
fn client_ip_prefers_cloudflare_then_the_first_forwarded_hop() {
    use axum::http::HeaderMap;
    let mut h = HeaderMap::new();
    assert_eq!(client_ip(&h), "unknown");

    h.insert("x-forwarded-for", "203.0.113.7, 10.0.0.1".parse().unwrap());
    assert_eq!(client_ip(&h), "203.0.113.7", "first hop only, not the whole chain");

    // Cloudflare's own header wins when both are present: it is set by the edge
    // and cannot be appended to by a client.
    h.insert("cf-connecting-ip", "198.51.100.9".parse().unwrap());
    assert_eq!(client_ip(&h), "198.51.100.9");

    // An empty header falls through rather than returning an empty bucket that
    // every visitor would share.
    let mut h2 = HeaderMap::new();
    h2.insert("x-forwarded-for", "".parse().unwrap());
    assert_eq!(client_ip(&h2), "unknown");
}

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

// ---- Session-cookie signing key ----
//
// The bug this pins, observed in production: `cookie::Key::generate()` at
// startup makes a NEW random key per process, and the session cookie's value is
// an HMAC over the session id under that key. Every restart therefore orphans
// every cookie already in a browser — the session row is still in Postgres and
// nothing can point at it — so a deploy signs out every logged-in user. The
// signing key must be a function of configuration, not of process lifetime.

use crate::auth::signing_key;

#[test]
fn signing_key_is_stable_for_one_secret() {
    // Two "processes" (two calls) must agree, or a restart signs everyone out.
    let first = signing_key("s3cret-configured-once");
    let second = signing_key("s3cret-configured-once");
    assert_eq!(
        first.signing(),
        second.signing(),
        "the same SESSION_SECRET must derive the same signing key in every process"
    );
}

#[test]
fn signing_key_changes_with_the_secret() {
    // Rotating the secret MUST rotate the key — otherwise a leaked secret could
    // not be retired.
    let a = signing_key("secret-one");
    let b = signing_key("secret-two");
    assert_ne!(a.signing(), b.signing(), "a different secret must give a different key");
    // And a key of a different length must not panic: the secret is SHA-256'd
    // to exactly 32 bytes first, because derive_from panics below that.
    let short = signing_key("x");
    assert_ne!(short.signing(), a.signing());
    assert_eq!(short.signing().len(), a.signing().len());
}
