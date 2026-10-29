//! GS-1 (guest identity) and GS-2 (the fold), driven through the REAL router.
//!
//! What these tests are actually pinning, in the standard's words: a guest is a real
//! identity rather than a shadow, the door closes on MAU, and folding a guest into a
//! new account moves the history without laundering anything — and refuses outright if
//! the schema has grown a column the fold does not handle.
//!
//! Run: DATABASE_URL=postgres://<role>:***@127.0.0.1:5499/streakforge_dev cargo test --test guest

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

fn test_config() -> Config {
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
        consent_version: 1,
        consent_required: true,
        guest_enabled: true,
        guest_mau_limit: 500,
        guest_mau_window_days: 30,
    }
}

async fn app(pool: PgPool) -> Router {
    app_with(pool, test_config()).await
}

async fn app_with(pool: PgPool, cfg: Config) -> Router {
    build_router(&cfg, pool).await.expect("the router builds")
}

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

/// Consent first (GS-3 gates the mint, which is the point of the ordering), then the
/// guest identity. Returns the cookie that now carries both.
async fn a_guest(app: &Router) -> String {
    let res = call(
        app,
        Method::POST,
        "/api/consent",
        None,
        Some(json!({"accept": true})),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let cookie = session_cookie(&res).expect("consent starts a session");

    let res = call(&app.clone(), Method::POST, "/api/guest", Some(&cookie), None).await;
    assert_eq!(res.status(), StatusCode::CREATED, "the guest is minted");
    session_cookie(&res).unwrap_or(cookie)
}

fn credentials(username: &str) -> Value {
    credentials_with(username, "correct-horse-battery")
}

fn credentials_with(username: &str, password: &str) -> Value {
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

// ── The identity ────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn a_guest_is_a_real_profile_with_no_credential(pool: PgPool) {
    let app = app(pool.clone()).await;
    let cookie = a_guest(&app).await;

    let (username, is_guest, hash, provider): (String, bool, Option<String>, String) =
        sqlx::query_as(
            "select username, is_guest, password_hash, provider from profiles where is_guest",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(username.starts_with("guest_"), "got {username}");
    assert!(is_guest);
    assert!(
        hash.is_none(),
        "a guest has no credential at all: an empty string is a hash some login path \
         might one day accept"
    );
    assert_eq!(provider, "local");

    // And the session knows who it is.
    let me = call(&app, Method::GET, "/api/auth/me", Some(&cookie), None).await;
    assert_eq!(me.status(), StatusCode::OK);
    let body = json_of(me).await;
    assert_eq!(body["is_guest"], true);
    assert_eq!(body["username"], username);
}

#[sqlx::test(migrations = "./migrations")]
async fn minting_a_guest_needs_consent_first(pool: PgPool) {
    let app = app(pool.clone()).await;
    let res = call(&app, Method::POST, "/api/guest", None, None).await;
    assert_eq!(
        res.status(),
        StatusCode::FORBIDDEN,
        "the guest identity is a cookie, so the gate has to precede it"
    );
    let rows: i64 = sqlx::query_scalar("select count(*) from profiles")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 0, "a refused mint writes no identity");
}

#[sqlx::test(migrations = "./migrations")]
async fn minting_twice_returns_the_same_identity(pool: PgPool) {
    let app = app(pool.clone()).await;
    let cookie = a_guest(&app).await;
    let first = json_of(call(&app, Method::GET, "/api/auth/me", Some(&cookie), None).await).await;

    let res = call(&app, Method::POST, "/api/guest", Some(&cookie), None).await;
    assert_eq!(res.status(), StatusCode::OK, "the second mint is not a 201");
    let second = json_of(res).await;
    assert_eq!(first["id"], second["id"], "one session, one identity");

    let rows: i64 = sqlx::query_scalar("select count(*) from profiles where is_guest")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_guest_cannot_sign_in_with_a_password(pool: PgPool) {
    let app = app(pool.clone()).await;
    let cookie = a_guest(&app).await;
    let username = json_of(call(&app, Method::GET, "/api/auth/me", Some(&cookie), None).await).await
        ["username"]
        .as_str()
        .unwrap()
        .to_string();

    let res = call(
        &app,
        Method::POST,
        "/api/auth/login",
        None,
        Some(credentials_with(&username, "anything-at-all")),
    )
    .await;
    assert_eq!(
        res.status(),
        StatusCode::UNAUTHORIZED,
        "there is nothing to sign in to"
    );
}

// ── Parity: a guest can do what a new account can ───────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn a_guest_can_log_and_the_streak_counts(pool: PgPool) {
    let app = app(pool.clone()).await;
    let cookie = a_guest(&app).await;

    let res = call(
        &app,
        Method::POST,
        "/api/logs",
        Some(&cookie),
        Some(json!({"note": "first", "kind": "habit"})),
    )
    .await;
    assert!(
        res.status().is_success(),
        "a guest logs like anyone else, got {}",
        res.status()
    );
    let body = json_of(res).await;
    assert_eq!(body["stats"]["today_count"], 1);
    assert_eq!(body["stats"]["current_streak"], 1);

    let logs: i64 = sqlx::query_scalar(
        "select count(*) from habit_logs l join profiles p on p.id = l.user_id where p.is_guest",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(logs, 1, "and the row is theirs");
}

// ── GS-2: the fold ──────────────────────────────────────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn registering_folds_the_guest_history_into_the_new_account(pool: PgPool) {
    let app = app(pool.clone()).await;
    let cookie = a_guest(&app).await;
    let guest_id: uuid::Uuid = sqlx::query_scalar("select id from profiles where is_guest")
        .fetch_one(&pool)
        .await
        .unwrap();
    for _ in 0..2 {
        let res = call(
            &app,
            Method::POST,
            "/api/logs",
            Some(&cookie),
            Some(json!({"note": "before registering"})),
        )
        .await;
        assert!(res.status().is_success(), "got {}", res.status());
    }

    let res = call(
        &app,
        Method::POST,
        "/api/auth/register",
        Some(&cookie),
        Some(credentials("convert")),
    )
    .await;
    assert_eq!(res.status(), StatusCode::CREATED);
    let account = json_of(res).await;
    let account_id = account["id"].as_str().unwrap().to_string();

    let moved: i64 =
        sqlx::query_scalar("select count(*) from habit_logs where user_id = $1::uuid")
            .bind(&account_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(moved, 2, "the history came with the account, not after it");

    let left: i64 = sqlx::query_scalar("select count(*) from habit_logs where user_id = $1")
        .bind(guest_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(left, 0, "and nothing still points at the guest");

    // Tenure, not score: the streak is computed from the moved logs, so day one is
    // not day zero — that is `merge.grants_tenure` on this site.
    let streak: i64 = sqlx::query_scalar("select current_streak::bigint from user_streak($1::uuid)")
        .bind(&account_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(streak, 1);

    // The tombstone: anonymised, not deleted, and it says where the history went.
    let (username, merged_into): (String, Option<uuid::Uuid>) =
        sqlx::query_as("select username, merged_into from profiles where id = $1")
            .bind(guest_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(username.starts_with("folded_"), "got {username}");
    assert_eq!(merged_into.map(|u| u.to_string()), Some(account_id.clone()));

    // And the fold is in the audit trail, naming the account.
    let audited: i64 = sqlx::query_scalar(
        "select count(*) from audit_log where action = 'guest.folded' and profile_id = $1::uuid",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(audited, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_fold_refuses_a_table_it_does_not_handle_and_creates_nothing(pool: PgPool) {
    let app = app(pool.clone()).await;
    let cookie = a_guest(&app).await;
    let guest_id: uuid::Uuid = sqlx::query_scalar("select id from profiles where is_guest")
        .fetch_one(&pool)
        .await
        .unwrap();
    call(
        &app,
        Method::POST,
        "/api/logs",
        Some(&cookie),
        Some(json!({"note": "keep me"})),
    )
    .await;

    // A migration lands that references profiles(id) and the fold has never heard of
    // it. This is the test the standard asks for by name: the check *is* the feature.
    sqlx::query("create table public.saved_searches (user_id uuid references public.profiles(id))")
        .execute(&pool)
        .await
        .unwrap();

    let res = call(
        &app,
        Method::POST,
        "/api/auth/register",
        Some(&cookie),
        Some(credentials("refusedfold")),
    )
    .await;
    assert_eq!(
        res.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "a fold that cannot finish must refuse loudly, not half-move a person's data"
    );

    let account: i64 = sqlx::query_scalar("select count(*) from profiles where username = 'refusedfold'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(account, 0, "and the account is not left behind without its history");

    let still_guest: i64 = sqlx::query_scalar("select count(*) from habit_logs where user_id = $1")
        .bind(guest_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(still_guest, 1, "nothing moved");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_retried_fold_moves_nothing_twice(pool: PgPool) {
    let app = app(pool.clone()).await;
    let cookie = a_guest(&app).await;
    let guest_id: uuid::Uuid = sqlx::query_scalar("select id from profiles where is_guest")
        .fetch_one(&pool)
        .await
        .unwrap();
    call(
        &app,
        Method::POST,
        "/api/logs",
        Some(&cookie),
        Some(json!({"note": "once"})),
    )
    .await;
    let account_id = json_of(
        call(
            &app,
            Method::POST,
            "/api/auth/register",
            Some(&cookie),
            Some(credentials("retryfold")),
        )
        .await,
    )
    .await["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Fold again by hand, as a retry or a double submit would.
    let mut conn = pool.acquire().await.unwrap();
    let report = streakforge_api::fold::fold_guest(&mut *conn, guest_id, account_id.parse().unwrap())
        .await
        .expect("a second fold is not an error");
    assert!(!report.folded, "the second fold reports that it did nothing");
    assert!(report.moved.is_empty());

    let moved: i64 = sqlx::query_scalar("select count(*) from habit_logs where user_id = $1::uuid")
        .bind(&account_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(moved, 1, "no double credit");
}

// ── The door closes on MAU ──────────────────────────────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn the_guest_door_closes_at_the_mau_limit(pool: PgPool) {
    let mut cfg = test_config();
    cfg.guest_mau_limit = 1;
    let app = app_with(pool.clone(), cfg).await;

    let first = a_guest(&app).await;
    call(
        &app,
        Method::POST,
        "/api/logs",
        Some(&first),
        Some(json!({"note": "counts as an active account"})),
    )
    .await;

    // A second visitor arrives: consent is fine, the guest door is not.
    let consent = call(
        &app,
        Method::POST,
        "/api/consent",
        None,
        Some(json!({"accept": true})),
    )
    .await;
    let second_cookie = session_cookie(&consent).unwrap();
    let res = call(
        &app,
        Method::POST,
        "/api/guest",
        Some(&second_cookie),
        None,
    )
    .await;
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    let status = call(&app, Method::GET, "/api/guest", None, None).await;
    let body = json_of(status).await;
    assert_eq!(body["at_cap"], true);
    assert_eq!(body["mau"], 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_guest_keeps_reading_at_the_cap_but_stops_writing(pool: PgPool) {
    let mut cfg = test_config();
    cfg.guest_mau_limit = 1;
    let app = app_with(pool.clone(), cfg).await;

    // Minting writes an audit row, and the audit row is an action, so the guest
    // counts as active the moment it exists: with a limit of 1 the door is already
    // at the cap before it does anything. That is the standard's definition, not a
    // leak — MAU counts accounts that acted, and a mint is an act.
    let cookie = a_guest(&app).await;
    let status = json_of(call(&app, Method::GET, "/api/guest", Some(&cookie), None).await).await;
    assert_eq!(status["at_cap"], true, "the mint filled the window by itself");

    let read = call(&app, Method::GET, "/api/stats", Some(&cookie), None).await;
    assert_eq!(
        read.status(),
        StatusCode::OK,
        "§1 keeps read access at the cap, and taking it away would be the opposite of grace"
    );

    let write = call(
        &app,
        Method::POST,
        "/api/logs",
        Some(&cookie),
        Some(json!({"note": "one too many"})),
    )
    .await;
    assert_eq!(write.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_door_can_be_closed_entirely(pool: PgPool) {
    let mut cfg = test_config();
    cfg.guest_enabled = false;
    let app = app_with(pool, cfg).await;

    let status = json_of(call(&app, Method::GET, "/api/guest", None, None).await).await;
    assert_eq!(status["enabled"], false);

    let consent = call(
        &app,
        Method::POST,
        "/api/consent",
        None,
        Some(json!({"accept": true})),
    )
    .await;
    let cookie = session_cookie(&consent).unwrap();
    let res = call(&app, Method::POST, "/api/guest", Some(&cookie), None).await;
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_zero_limit_means_the_door_is_shut(pool: PgPool) {
    // §1: "`0` disables guest access entirely" — not "unlimited", which is how a
    // zero limit reads if nobody says otherwise.
    let mut cfg = test_config();
    cfg.guest_mau_limit = 0;
    let app = app_with(pool.clone(), cfg).await;

    let status = json_of(call(&app, Method::GET, "/api/guest", None, None).await).await;
    assert_eq!(status["enabled"], false);
    assert_eq!(status["at_cap"], false, "at_cap is about the cap, not the switch");
}

/// The cage rows are the guest's own tenure, so they fold; a vote about someone
/// else's content does not, because moving it would let one person cast the same
/// judgement twice under two names.
///
/// The attestation and the flag are inserted directly: both endpoints require trust
/// level 1 and a guest is level 0 by definition, so the API cannot produce these
/// rows. That is the point of testing the fold itself — it must handle every column
/// that references `profiles(id)`, reachable or not.
#[sqlx::test(migrations = "./migrations")]
async fn the_fold_moves_cage_tenure_and_leaves_votes_with_the_tombstone(pool: PgPool) {
    let app = app(pool.clone()).await;
    let cookie = a_guest(&app).await;
    let guest_id: uuid::Uuid = sqlx::query_scalar("select id from profiles where is_guest")
        .fetch_one(&pool)
        .await
        .unwrap();

    // The guest's own cage: a lock and a check with a peer-verified hour.
    let res = call(&app, Method::POST, "/api/lock", Some(&cookie), None).await;
    assert_eq!(res.status(), StatusCode::CREATED);
    let res = call(
        &app,
        Method::POST,
        "/api/cage-check",
        Some(&cookie),
        Some(json!({"note": "still locked"})),
    )
    .await;
    assert_eq!(res.status(), StatusCode::CREATED);
    let check_id: i64 = sqlx::query_scalar("select id from cage_checks where user_id = $1")
        .bind(guest_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("update cage_checks set status = 'verified', claimed_secs = 7200 where id = $1")
        .bind(check_id)
        .execute(&pool)
        .await
        .unwrap();

    // Someone else's check, attested by the guest, and a flag the guest raised.
    let other: uuid::Uuid =
        sqlx::query_scalar("insert into profiles (username) values ('somebody') returning id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let other_check: i64 = sqlx::query_scalar(
        "insert into cage_checks (user_id, seal_code, claimed_secs) values ($1, 'deadbeef', 60) returning id",
    )
    .bind(other)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "insert into cage_attestations (check_id, attester_id, verdict) values ($1, $2, 'confirm')",
    )
    .bind(other_check)
    .bind(guest_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("insert into flags (target_kind, target_id, reporter_id, reason) values ('cage_check', $1, $2, 'spam')")
        .bind(other_check)
        .bind(guest_id)
        .execute(&pool)
        .await
        .unwrap();

    let res = call(
        &app,
        Method::POST,
        "/api/auth/register",
        Some(&cookie),
        Some(credentials("cageconvert")),
    )
    .await;
    assert_eq!(res.status(), StatusCode::CREATED, "the fold must not refuse");
    let account_id = json_of(res).await["id"].as_str().unwrap().to_string();

    let checks: i64 = sqlx::query_scalar("select count(*) from cage_checks where user_id = $1::uuid")
        .bind(&account_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(checks, 1, "the cage check came with the account");

    let attests: i64 =
        sqlx::query_scalar("select count(*) from cage_attestations where attester_id = $1::uuid")
            .bind(&account_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(attests, 1, "and so did the attestation: it counts towards tenure");

    let flags: i64 = sqlx::query_scalar("select count(*) from flags where reporter_id = $1")
        .bind(guest_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        flags, 1,
        "the flag stays with the identity that raised it, so one person cannot vote twice"
    );

    // The hours the guest earned are now the account's, verified included.
    let (verified,): (i64,) = sqlx::query_as("select verified_secs::bigint from cage_check_hours($1::uuid)")
        .bind(&account_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(verified, 7_200);
}
