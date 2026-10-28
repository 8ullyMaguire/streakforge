//! GS-3 — the consent gate, driven through the REAL router.
//!
//! Why through the router: the gate is middleware, and the thing that can go
//! wrong is *which requests it lets through*, not what any one handler does with
//! a session value. A function-level test would call `accepted()` and assert the
//! same assumption the middleware makes; these tests send HTTP and read the JSON,
//! so a route added outside the layer is caught by the suite rather than by a
//! person noticing that one endpoint forgot to ask.
//!
//! Run: DATABASE_URL=postgres://<role>:***@127.0.0.1:5499/streakforge_dev cargo test --test consent

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use streakforge_api::auth::{compute_challenge_proof, session_cookie_name};
use streakforge_api::config::Config;
use streakforge_api::router::build_router;
use tower::ServiceExt;

fn test_config(version: i32) -> Config {
    Config {
        database_url: "unused-by-the-router".into(),
        redis_url: "unused-by-the-router".into(),
        session_secret: "test-secret-not-a-real-one-0123456789abcdef".into(),
        public_url: "http://127.0.0.1:8787".into(),
        secure_cookies: false,
        web_build_dir: "./does-not-exist-in-tests".into(),
        manifestos_dir: "../manifestos".into(),
        migrations_dir: "./migrations".into(),
        session_ttl_days: 30,
        wlw_daily_limit: 5,
        consent_version: version,
        consent_required: true,
        guest_enabled: true,
        guest_mau_limit: 500,
        guest_mau_window_days: 30,
    }
}

async fn app(pool: PgPool, version: i32) -> Router {
    build_router(&test_config(version), pool)
        .await
        .expect("the router builds")
}

/// One request. `cookie` is the `name=value` pair a browser would send back.
async fn call(
    app: &Router,
    method: Method,
    path: &str,
    cookie: Option<&str>,
    body: Option<Value>,
) -> axum::response::Response {
    let mut req = Request::builder().method(method).uri(path);
    if let Some(c) = cookie {
        req = req.header(header::COOKIE, c);
    }
    let req = match body {
        Some(v) => req
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(v.to_string()))
            .unwrap(),
        None => req.body(Body::empty()).unwrap(),
    };
    app.clone().oneshot(req).await.expect("the router answers")
}

async fn json_of(res: axum::response::Response) -> Value {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

/// The session cookie from a response, as a client would send it back.
fn session_cookie(res: &axum::response::Response) -> Option<String> {
    let name = session_cookie_name(false);
    res.headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|v| v.starts_with(&format!("{name}=")))
        .and_then(|v| v.split(';').next())
        .map(|v| v.to_string())
}

/// Accept consent on a fresh session and return the cookie that carries it.
async fn consenting_cookie(app: &Router) -> String {
    let res = call(
        app,
        Method::POST,
        "/api/consent",
        None,
        Some(json!({"accept": true, "categories": ["analytics"]})),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK, "consent is recorded");
    session_cookie(&res).expect("accepting consent starts a session")
}

/// The sign-in form, challenge included: the handler refuses a form without a
/// proof, so a test that omits it would prove the challenge and not the gate.
fn credentials(username: &str) -> Value {
    let password = "correct-horse-battery";
    // 32 hex characters, and a form that has been open for five seconds: the
    // handler refuses a form opened under 3s ago (a bot) or over 10min ago (a
    // stale tab), so a test with `form_opened_at: 0` would be testing that rule.
    let nonce = "0123456789abcdef0123456789abcdef";
    let opened = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
        - 5_000;
    json!({
        "username": username,
        "password": password,
        "form_opened_at": opened,
        "website": "",
        "challenge_nonce": nonce,
        "challenge_proof": compute_challenge_proof(nonce, username, password),
    })
}

// ── Reads stay open ─────────────────────────────────────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn a_visitor_reads_the_board_without_consenting(pool: PgPool) {
    let app = app(pool, 1).await;
    for path in ["/api/feed", "/api/kpi", "/api/leaderboard/weekly"] {
        let res = call(&app, Method::GET, path, None, None).await;
        assert_eq!(
            res.status(),
            StatusCode::OK,
            "{path} must stay readable: the gate covers mutations, and a public board that \
             hides behind a cookie banner is not public"
        );
    }
}

// ── Mutations do not ────────────────────────────────────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn registration_is_refused_until_consent_is_given(pool: PgPool) {
    let app = app(pool.clone(), 1).await;
    let res = call(
        &app,
        Method::POST,
        "/api/auth/register",
        None,
        Some(credentials("noconsent")),
    )
    .await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    let body = json_of(res).await;
    assert_eq!(
        body["consent"], "required",
        "the refusal has to be machine-readable: a client that only gets a prose \
         error cannot open the right dialog, and one that guesses shows the wrong one"
    );
    assert_eq!(body["version"], 1);

    let rows: i64 = sqlx::query_scalar("select count(*) from profiles")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 0, "a refused mutation must not have written anything");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_refusal_is_honoured_and_nothing_is_persisted(pool: PgPool) {
    let app = app(pool.clone(), 1).await;
    let res = call(
        &app,
        Method::POST,
        "/api/consent",
        None,
        Some(json!({"accept": false})),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK, "a refusal is a decision, not an error");
    let body = json_of(res).await;
    assert_eq!(body["consent"], "refused");

    // Still refused afterwards, and the site still readable.
    let reg = call(
        &app,
        Method::POST,
        "/api/auth/register",
        None,
        Some(credentials("refuser")),
    )
    .await;
    assert_eq!(reg.status(), StatusCode::FORBIDDEN);
    let feed = call(&app, Method::GET, "/api/feed", None, None).await;
    assert_eq!(feed.status(), StatusCode::OK);

    let accepted: i64 =
        sqlx::query_scalar("select count(*) from audit_log where action = 'consent.accepted'")
            .fetch_one(&pool)
            .await
            .unwrap();
    let refused: i64 =
        sqlx::query_scalar("select count(*) from audit_log where action = 'consent.refused'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((accepted, refused), (0, 1), "the audit says what happened");
    let rows: i64 = sqlx::query_scalar("select count(*) from profiles")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 0);
}

// ── Accepting, and what survives the browser ────────────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn consent_lets_registration_through_and_stamps_the_account(pool: PgPool) {
    let app = app(pool.clone(), 1).await;
    let cookie = consenting_cookie(&app).await;

    let res = call(
        &app,
        Method::POST,
        "/api/auth/register",
        Some(&cookie),
        Some(credentials("consented")),
    )
    .await;
    assert_eq!(res.status(), StatusCode::CREATED);

    let (consented, version): (bool, Option<i32>) = sqlx::query_as(
        "select consented_at is not null, consent_version from profiles where username = $1",
    )
    .bind("consented")
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(consented, "the account carries the consent, not just the browser");
    assert_eq!(version, Some(1));

    // And the record is in the audit log, linked to the account.
    let audited: i64 = sqlx::query_scalar(
        "select count(*) from audit_log a join profiles p on p.id = a.profile_id
         where p.username = $1 and a.action = 'consent.accepted'",
    )
    .bind("consented")
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(audited >= 1, "the audit row names the account");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_consenting_account_signs_in_on_a_device_that_was_never_asked(pool: PgPool) {
    let app = app(pool.clone(), 1).await;
    let cookie = consenting_cookie(&app).await;
    let reg = call(
        &app,
        Method::POST,
        "/api/auth/register",
        Some(&cookie),
        Some(credentials("returning")),
    )
    .await;
    assert_eq!(reg.status(), StatusCode::CREATED);

    // A different browser: no cookie at all. The account's own record is what
    // lets this through — otherwise the gate relocks on every device change.
    let login = call(
        &app,
        Method::POST,
        "/api/auth/login",
        None,
        Some(credentials("returning")),
    )
    .await;
    assert_eq!(
        login.status(),
        StatusCode::OK,
        "signing in must not require a fresh consent the account already gave"
    );
    let fresh = session_cookie(&login).expect("a session cookie comes back");

    // And that session now counts as consenting for the mutating routes.
    let state = call(&app, Method::GET, "/api/consent", Some(&fresh), None).await;
    let body = json_of(state).await;
    assert_eq!(body["accepted"], true);
    assert_eq!(body["version"], 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn an_account_that_never_consented_cannot_sign_in_without_asking(pool: PgPool) {
    let app = app(pool.clone(), 1).await;
    let cookie = consenting_cookie(&app).await;
    let reg = call(
        &app,
        Method::POST,
        "/api/auth/register",
        Some(&cookie),
        Some(credentials("legacy")),
    )
    .await;
    assert_eq!(reg.status(), StatusCode::CREATED);

    // Simulate an account created before this gate existed.
    sqlx::query("update profiles set consented_at = null, consent_version = null where username = $1")
        .bind("legacy")
        .execute(&pool)
        .await
        .unwrap();

    let login = call(
        &app,
        Method::POST,
        "/api/auth/login",
        None,
        Some(credentials("legacy")),
    )
    .await;
    assert_eq!(login.status(), StatusCode::FORBIDDEN);
    let body = json_of(login).await;
    assert_eq!(body["consent"], "required");
}

// ── Versioning, categories, exemptions ──────────────────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn bumping_the_version_asks_everyone_again(pool: PgPool) {
    let v1 = app(pool.clone(), 1).await;
    let cookie = consenting_cookie(&v1).await;
    let ok = call(
        &v1,
        Method::POST,
        "/api/lock",
        Some(&cookie),
        Some(json!({})),
    )
    .await;
    assert_ne!(
        ok.status(),
        StatusCode::FORBIDDEN,
        "a consenting session reaches the handler (whatever the handler then says)"
    );

    // The same browser against a site that now asks for version 2.
    let v2 = app(pool, 2).await;
    let res = call(&v2, Method::POST, "/api/lock", Some(&cookie), Some(json!({}))).await;
    assert_eq!(
        res.status(),
        StatusCode::FORBIDDEN,
        "changing what is collected is a new question, not an old answer"
    );
    assert_eq!(json_of(res).await["version"], 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn only_essential_is_on_unless_a_category_was_accepted(pool: PgPool) {
    let app = app(pool, 1).await;

    let bare = call(
        &app,
        Method::POST,
        "/api/consent",
        None,
        Some(json!({"accept": true})),
    )
    .await;
    let cookie = session_cookie(&bare).expect("a session");
    let body = json_of(bare).await;
    assert_eq!(body["categories"]["essential"], true);
    assert_eq!(body["categories"]["analytics"], false);
    assert_eq!(body["categories"]["personalisation"], false);

    let state = call(&app, Method::GET, "/api/consent", Some(&cookie), None).await;
    let body = json_of(state).await;
    assert_eq!(body["categories"]["analytics"], false);
}

#[sqlx::test(migrations = "./migrations")]
async fn an_unknown_category_is_not_recorded(pool: PgPool) {
    let app = app(pool.clone(), 1).await;
    let res = call(
        &app,
        Method::POST,
        "/api/consent",
        None,
        Some(json!({"accept": true, "categories": ["analytics", "advertising"]})),
    )
    .await;
    let body = json_of(res).await;
    assert_eq!(body["categories"]["analytics"], true);
    assert!(
        body["categories"].get("advertising").is_none(),
        "a category this site does not have was not accepted, and must not be stored"
    );
    let detail: Value = sqlx::query_scalar(
        "select detail from audit_log where action = 'consent.accepted' order by id desc limit 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(detail["categories"], json!(["analytics"]));
}

#[sqlx::test(migrations = "./migrations")]
async fn ending_a_session_never_needs_permission(pool: PgPool) {
    let app = app(pool, 1).await;
    let res = call(&app, Method::POST, "/api/auth/logout", None, None).await;
    assert_ne!(
        res.status(),
        StatusCode::FORBIDDEN,
        "a logout that asks for consent first is a trap, not a gate"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn the_gate_can_be_turned_off_without_a_rebuild(pool: PgPool) {
    let mut cfg = test_config(1);
    cfg.consent_required = false;
    let app = build_router(&cfg, pool).await.expect("the router builds");

    let res = call(
        &app,
        Method::POST,
        "/api/auth/register",
        None,
        Some(credentials("switchoff")),
    )
    .await;
    assert_eq!(
        res.status(),
        StatusCode::CREATED,
        "CONSENT_REQUIRED=0 is how a deployment answers an incident; it must not need a build"
    );
}
