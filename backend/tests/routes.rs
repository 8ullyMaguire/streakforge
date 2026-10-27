//! End-to-end tests through the REAL router.
//!
//! WHY THIS FILE EXISTS: every public page here reads a SQL function whose OUT
//! list SQLx decodes by position, and this project has already served two 500s
//! from exactly that (a numeric into f64, an INT4 into i64) with the whole suite
//! green. `integration.rs` asserts against the same assumptions the handlers
//! make, so it cannot catch a handler/JSON mismatch. These tests go through
//! `build_router`, send HTTP requests, and read the JSON body — the only shape
//! that notices.
//!
//! Run: DATABASE_URL=postgres://<role>:<pw>@127.0.0.1:5499/streakforge_dev cargo test --test routes

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use sqlx::PgPool;
use streakforge_api::auth::compute_challenge_proof;
use streakforge_api::config::Config;
use streakforge_api::router::build_router;
use tower::ServiceExt;

/// A config that does not depend on the environment, so a test cannot pass or
/// fail because of a stray exported variable. `secure_cookies` is false here on
/// purpose: the `__Host-` cookie name requires Secure, and a test drives the
/// router over http, so the plain name is the one that comes back. Production
/// sets it to 1 and the cookie prefix is verified against the live site.
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
        // The gate is on in these tests: the harness below accepts consent
        // before it mutates, which is also what proves the gate is passable.
        consent_version: 1,
        consent_required: true,
    }
}

async fn app(pool: PgPool) -> Router {
    build_router(&test_config(), pool)
        .await
        .expect("the router builds")
}

async fn json_of(res: axum::response::Response) -> serde_json::Value {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
}

/// The `Set-Cookie` value, reduced to the `name=value` pair a client would send
/// back.
fn session_cookie(res: &axum::response::Response) -> String {
    res.headers()
        .get("set-cookie")
        .expect("the response sets a session cookie")
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string()
}

/// A completed auth form: the honeypot is empty, the form was "open" long enough
/// to look human, and the proof is the real one. Bypassing any of the three is a
/// 400, which is why the probe scripts have to reproduce them too.
fn auth_body(username: &str, password: &str) -> serde_json::Value {
    let nonce = "0123456789abcdef0123456789abcdef";
    let opened = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64)
        - 4_000;
    serde_json::json!({
        "username": username,
        "password": password,
        "form_opened_at": opened,
        "website": "",
        "challenge_nonce": nonce,
        "challenge_proof": compute_challenge_proof(nonce, username, password),
    })
}

/// Give consent on a fresh session, the way the browser does before it submits
/// the form: the toast is answered on the first page, not at the submit button.
///
/// Doing it here rather than turning the gate off in `test_config` keeps the rest
/// of this file honest — every mutation below goes through the real layer.
async fn consent(app: &Router) -> String {
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/consent")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"accept":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK, "consent is recorded");
    session_cookie(&res)
}

async fn register(app: &Router, username: &str) -> String {
    let cookie = consent(app).await;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(
                    auth_body(username, "correct-horse-battery").to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED, "register must succeed");
    session_cookie(&res)
}

async fn get(app: &Router, path: &str, cookie: Option<&str>) -> axum::response::Response {
    let mut req = Request::builder().method("GET").uri(path);
    if let Some(c) = cookie {
        req = req.header("cookie", c);
    }
    app.clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

// ---- the surface answers, and answers JSON ---------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn health_reports_the_database(pool: PgPool) {
    // A health check that cannot fail is not a check: before this route existed,
    // GET /health returned 200 on a broken database because the SPA fallback
    // served index.html.
    let app = app(pool).await;
    // Both paths, because `/health` is what a monitor guesses first and it used to
    // answer 200 with the SPA shell.
    for path in ["/api/health", "/health"] {
        let res = get(&app, path, None).await;
        assert_eq!(
            res.status(),
            StatusCode::OK,
            "{path} must be a real health check"
        );
        let body = json_of(res).await;
        assert_eq!(body["status"], "ok", "{path}");
        assert_eq!(body["database"], "ok", "{path}");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn health_goes_503_when_the_database_is_gone(pool: PgPool) {
    // The half that makes the other half mean anything: a health check that cannot
    // fail is a status code describing nothing. Closing the pool out from under the
    // router is the cheapest faithful way to break the database for one request
    // without touching the scratch database's schema.
    let app = app(pool.clone()).await;
    assert_eq!(
        get(&app, "/api/health", None).await.status(),
        StatusCode::OK
    );

    pool.close().await;

    let res = get(&app, "/api/health", None).await;
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = json_of(res).await;
    assert_eq!(body["status"], "degraded");
    assert_eq!(body["database"], "unreachable");
}

#[sqlx::test(migrations = "./migrations")]
async fn an_unknown_api_path_404s_rather_than_serving_the_app(pool: PgPool) {
    let res = get(&app(pool).await, "/api/does-not-exist", None).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// ---- the shapes that have broken in production -----------------------------

#[sqlx::test(migrations = "./migrations")]
async fn every_leaderboard_period_decodes_and_answers(pool: PgPool) {
    // Both historical outages were a leaderboard: the function's OUT list moved
    // and the Rust tuple did not. Four periods, each through the handler.
    let app = app(pool).await;
    for period in ["daily", "weekly", "monthly", "alltime"] {
        let res = get(&app, &format!("/api/leaderboard/{period}"), None).await;
        assert_eq!(
            res.status(),
            StatusCode::OK,
            "leaderboard/{period} must not 500"
        );
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn the_public_feed_has_no_note_field_while_the_note_survives_in_the_table(pool: PgPool) {
    // Item 01, asserted at the layer the user actually receives. The asymmetry is
    // the proof: the row still holds the note, and the endpoint cannot produce
    // it. An empty table would prove nothing.
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username, created_at) values ('feedprobe', now() - interval '2 days') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "insert into habit_logs (user_id, kind, note) values ($1, 'habit', 'a private confession')",
    )
    .bind(uid)
    .execute(&pool)
    .await
    .unwrap();

    let res = get(&app(pool.clone()).await, "/api/feed?limit=10", None).await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json_of(res).await;
    let items = body["items"]
        .as_array()
        .expect("the feed returns an items array");
    assert_eq!(items.len(), 1, "the seeded log must appear");
    assert!(
        items[0].get("note").is_none(),
        "the anonymous feed must not carry a note field at all: {items:?}"
    );
    assert_eq!(items[0]["username"], "feedprobe");
    assert!(
        items[0].get("logged_at").is_some(),
        "the feed still says when"
    );

    let notes: i64 = sqlx::query_scalar("select count(*) from habit_logs where note is not null")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        notes, 1,
        "the note is still in the table — privacy is about the endpoint"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn today_counts_decode_positionally(pool: PgPool) {
    let res = get(&app(pool).await, "/api/today", None).await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json_of(res).await;
    for field in ["denials_today", "wastes_today", "loggers_today"] {
        assert!(
            body[field].is_number(),
            "today_counts must give a number for {field}"
        );
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn the_kpi_page_decodes_every_column(pool: PgPool) {
    // kpi_totals has nine columns and a double precision in the middle of them,
    // which is the exact shape that 500'd once already.
    let res = get(&app(pool).await, "/api/kpi", None).await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json_of(res).await;
    assert!(
        body["total_wasted"].is_number() || body["totals"]["total_wasted"].is_number(),
        "KPI payload: {body}"
    );
}

// ---- the session, end to end -----------------------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn register_then_me_round_trips_a_session(pool: PgPool) {
    let app = app(pool).await;
    let cookie = register(&app, "roundtrip").await;

    let res = get(&app, "/api/auth/me", Some(&cookie)).await;
    assert_eq!(
        res.status(),
        StatusCode::OK,
        "the cookie from register must authenticate"
    );
    let body = json_of(res).await;
    assert_eq!(body["username"], "roundtrip");

    // And without it, the same route says who you are: nobody.
    let res = get(&app, "/api/auth/me", None).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_persistent_guesser_waits_longer_than_a_typo(pool: PgPool) {
    // Item 06 asked for an exponential backoff, and the plan shipped a flat
    // 15-minute window instead. This is the wiring check for the fix: failures in
    // the table must lengthen the `Retry-After` the client receives.
    let app = app(pool.clone()).await;
    register(&app, "persistent").await;

    let mut wrong = auth_body("persistent", "not-the-password");
    wrong["challenge_proof"] = serde_json::json!(""); // not reached: the throttle runs first
    let attempt = |app: Router, body: serde_json::Value| async move {
        app.oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
    };

    // Five wrong guesses: allowed, and each one recorded.
    for i in 0..5 {
        let res = attempt(app.clone(), auth_body("persistent", "wrong")).await;
        assert_eq!(
            res.status(),
            StatusCode::UNAUTHORIZED,
            "guess {i} is a plain 401"
        );
    }
    // The sixth is refused, and says how long to wait.
    let res = attempt(app.clone(), auth_body("persistent", "wrong")).await;
    assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);
    let short: i64 = res
        .headers()
        .get("retry-after")
        .expect("a refusal must carry Retry-After")
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!(
        short >= 1,
        "never zero: an immediate retry would be refused again"
    );

    // Now make the day persistent — nine failures inside 24 hours, which is past
    // any plausible run of typos — and the same refusal must quote a longer wait.
    sqlx::query(
        "insert into login_attempts (username, ip)
         select 'persistent', 'unknown' from generate_series(1, 4)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let res = attempt(app.clone(), auth_body("persistent", "wrong")).await;
    assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);
    let long: i64 = res
        .headers()
        .get("retry-after")
        .expect("still refused")
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!(
        long >= 1_800,
        "nine failures in a day must escalate past the 15-minute window, got {long}"
    );
    assert!(
        long > short,
        "the escalation has to be visible to the client: {short} -> {long}"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn stats_reports_the_budget_and_the_decay_countdown(pool: PgPool) {
    let app = app(pool).await;
    let cookie = register(&app, "budgetprobe").await;

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/logs")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(r#"{"kind":"habit","note":"probe"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        StatusCode::CREATED,
        "a first waste of the day must be allowed"
    );

    let res = get(&app, "/api/stats", Some(&cookie)).await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json_of(res).await;
    assert_eq!(
        body["daily_limit"], 5,
        "the limit the UI shows is the operator's"
    );
    assert_eq!(body["remaining_today"], 4);
    assert_eq!(
        body["devotion"]["days_until_decay"], 30,
        "a fresh log is 30 days from decay"
    );
    assert_eq!(
        body["devotion"]["wlw"], 1,
        "one day wasted, whatever the row count"
    );
    // The author's own note comes back here — that half of item 01 must not
    // regress while the public half stays closed.
    assert_eq!(body["recent_logs"][0]["note"], "probe");

    // Item 15's clock — the assertion whose absence let the dashboard say
    // "4 of 5 left today" for most of every day. The two countdowns answer
    // different questions and must not share an answer:
    assert_eq!(
        body["can_log"], true,
        "4 of 5 left means another log is allowed right now"
    );
    assert_eq!(
        body["next_allowed_in_secs"], 0,
        "the lock has nothing to wait for, so ITS countdown is legitimately 0"
    );
    let reset = body["secs_until_reset"]
        .as_i64()
        .expect("secs_until_reset must be a number");
    assert!(
        reset > 0 && reset <= 86_400,
        "the day's allowance still has a clock ({reset}s until UTC midnight), even while a log is allowed"
    );
}
