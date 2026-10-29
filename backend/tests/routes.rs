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
        guest_enabled: true,
        guest_mau_limit: 500,
        guest_mau_window_days: 30,
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

/// Sign an existing account back in. The login form carries the same three anti-bot
/// fields as registration (`auth_body`), so a hand-written body is a 400 — which is what
/// the first version of the deck test did.
async fn login(app: &Router, username: &str) -> String {
    let cookie = consent(app).await;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(
                    auth_body(username, "correct-horse-battery").to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK, "login must succeed");
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
async fn the_public_feed_carries_the_note_until_the_author_opts_out(pool: PgPool) {
    // THIS TEST USED TO ASSERT THE FEED HAD NO NOTE FIELD. The owner reversed that
    // policy on 2026-10-29 ("confessions/logs are public by default, user can
    // choose to make them private through a setting"), so the assertion is now the
    // switch, at the layer the user receives it: published by default, gone from
    // the endpoint when `logs_public` is false — while the row itself survives.
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username, created_at) values ('feedprobe', now() - interval '2 days') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "insert into habit_logs (user_id, kind, note) values ($1, 'habit', 'a public confession')",
    )
    .bind(uid)
    .execute(&pool)
    .await
    .unwrap();

    let app = app(pool.clone()).await;
    let res = get(&app, "/api/feed?limit=10", None).await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json_of(res).await;
    let items = body["items"]
        .as_array()
        .expect("the feed returns an items array");
    assert_eq!(items.len(), 1, "the seeded log must appear");
    assert_eq!(
        items[0]["note"], "a public confession",
        "a confession is public by default: {items:?}"
    );
    assert_eq!(items[0]["kind"], "habit");
    assert_eq!(items[0]["username"], "feedprobe");

    sqlx::query("update profiles set logs_public = false where id = $1")
        .bind(uid)
        .execute(&pool)
        .await
        .unwrap();
    let body = json_of(get(&app, "/api/feed?limit=10", None).await).await;
    assert_eq!(
        body["items"].as_array().unwrap().len(),
        0,
        "the setting takes the row out of the public feed"
    );

    let notes: i64 = sqlx::query_scalar("select count(*) from habit_logs where note is not null")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        notes, 1,
        "the note is still in the table — the setting governs publication, not storage"
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

/// POST one log of `kind` as the session in `cookie`.
async fn log_kind(app: &Router, cookie: &str, kind: &str) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/logs")
                .header("content-type", "application/json")
                .header("cookie", cookie)
                .body(Body::from(format!(r#"{{"kind":"{kind}"}}"#)))
                .unwrap(),
        )
        .await
        .unwrap()
}

/// One-way exclusivity, asserted through the router — i.e. against the exact
/// payload the dashboard button reads.
///
/// The owner's report was behavioural: "I should be able to click waste load
/// right after denial." A disabled button is not a styling problem — it is
/// `GET /api/stats` answering `can_log: false` with `blocked_by_exclusivity:
/// true`, because the habit payload mirrors `check_rate_limits()` and that
/// function used to refuse a waste for the rest of a day that held a denial.
/// So this test asserts the JSON the button sees AND the 201 the click makes,
/// and then that the other direction is still refused.
#[sqlx::test(migrations = "./migrations")]
async fn a_denial_blocks_the_next_denial_but_never_the_waste_button(pool: PgPool) {
    let app = app(pool).await;
    let cookie = register(&app, "onewayprobe").await;

    let first = log_kind(&app, &cookie, "denial").await;
    assert_eq!(
        first.status(),
        StatusCode::CREATED,
        "the day's one denial must log"
    );

    // What the WASTE A LOAD button sees one second later.
    let res = get(&app, "/api/stats", Some(&cookie)).await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json_of(res).await;
    assert_eq!(
        body["blocked_by_exclusivity"], false,
        "a denial must not mark the waste as exclusive-blocked"
    );
    assert_eq!(
        body["can_log"], true,
        "can_log drives the button's disabled attribute — it must be true after a denial"
    );
    assert_eq!(
        body["remaining_today"], 5,
        "a denial does not spend a slot of the waste budget"
    );
    assert_eq!(
        body["next_allowed_in_secs"], 0,
        "nothing to wait for: the waste is allowed right now"
    );

    let waste = log_kind(&app, &cookie, "habit").await;
    assert_eq!(
        waste.status(),
        StatusCode::CREATED,
        "a load wasted after a reported denial is the outcome the site counts"
    );

    // The other direction is the one the owner kept: a whiteboi who cums cannot
    // claim he was denied.
    let refused = log_kind(&app, &cookie, "denial").await;
    assert_eq!(
        refused.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "a denial after a waste on the same UTC day must be refused"
    );
    let body = json_of(refused).await;
    let msg = body["error"].as_str().unwrap_or_default();
    assert!(
        msg.contains("load wasted today"),
        "the refusal has to name the rule that fired: {msg}"
    );
}

/// POST a JSON body as `cookie`; returns the raw response so callers can assert
/// status and body separately.
async fn post_json(app: &Router, path: &str, cookie: &str, body: &str) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", "application/json")
                .header("cookie", cookie)
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn send(
    app: &Router,
    method: &str,
    path: &str,
    cookie: &str,
    body: Option<&str>,
) -> axum::response::Response {
    let mut req = Request::builder()
        .method(method)
        .uri(path)
        .header("cookie", cookie);
    if body.is_some() {
        req = req.header("content-type", "application/json");
    }
    app.clone()
        .oneshot(req.body(Body::from(body.unwrap_or("").to_string())).unwrap())
        .await
        .unwrap()
}

/// Give an account trust level 1 the honest way: backdate it a day and let it log
/// three times. The level is computed from those rows, so nothing here grants it.
async fn make_trust_1(pool: &PgPool, username: &str) {
    sqlx::query("update profiles set created_at = now() - interval '2 days' where username = $1")
        .bind(username)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query(
        "insert into habit_logs (user_id, kind)
         select p.id, 'habit' from profiles p, generate_series(1, 3) where p.username = $1",
    )
    .bind(username)
    .execute(pool)
    .await
    .expect("three logs for the trust floor");
}

/// The one-click landing flow: one POST writes the log, the SAME fifteen minutes
/// then carry the time-to-cum number and the undo.
#[sqlx::test(migrations = "./migrations")]
async fn a_log_can_be_given_a_duration_and_undone_inside_the_window(pool: PgPool) {
    let app = app(pool.clone()).await;
    let cookie = register(&app, "oneclick").await;

    let res = post_json(&app, "/api/logs", &cookie, r#"{"kind":"habit"}"#).await;
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = json_of(res).await;
    assert_eq!(body["stats"]["today_count"], 1);
    assert_eq!(
        body["stats"]["time_to_cum"]["samples"], 0,
        "a one-click log carries no duration yet"
    );

    // The id the UI patches: it comes from the owner's own recent logs.
    let res = get(&app, "/api/stats", Some(&cookie)).await;
    let body = json_of(res).await;
    let log_id = body["recent_logs"][0]["id"].as_i64().expect("the log id");

    let res = send(
        &app,
        "PATCH",
        &format!("/api/logs/{log_id}"),
        &cookie,
        Some(r#"{"time_to_cum_secs":900}"#),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK, "the duration is accepted in-window");
    let body = json_of(res).await;
    assert_eq!(body["stats"]["time_to_cum"]["samples"], 1);
    assert_eq!(body["stats"]["time_to_cum"]["avg_secs"], 900.0);

    let res = send(&app, "DELETE", &format!("/api/logs/{log_id}"), &cookie, None).await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT, "undo inside the window");
    let res = get(&app, "/api/stats", Some(&cookie)).await;
    let body = json_of(res).await;
    assert_eq!(body["today_count"], 0, "the row is gone, and so is its budget use");

    // Past the window both writes are refused, and the message names the limit.
    let res = post_json(&app, "/api/logs", &cookie, r#"{"kind":"habit"}"#).await;
    let body = json_of(res).await;
    let log_id = body["stats"]["recent_logs"][0]["id"].as_i64().unwrap();
    // `created_at` is the clock the undo window reads (0018): a log's age is how
    // long ago it was WRITTEN, not how long ago the day it describes was. Moving
    // only `logged_at` no longer expires the window, and this test is the proof.
    sqlx::query(
        "update habit_logs set logged_at = now() - interval '20 minutes', created_at = now() - interval '20 minutes' where id = $1",
    )
        .bind(log_id)
        .execute(&pool)
        .await
        .unwrap();
    let res = send(
        &app,
        "PATCH",
        &format!("/api/logs/{log_id}"),
        &cookie,
        Some(r#"{"time_to_cum_secs":900}"#),
    )
    .await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    let res = send(&app, "DELETE", &format!("/api/logs/{log_id}"), &cookie, None).await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

/// Cage checks end to end: locked → a check with a seal → two peers verify → the
/// hours are verified hours. Also the three refusals that make it mean something.
#[sqlx::test(migrations = "./migrations")]
async fn a_cage_check_needs_a_lock_and_two_peers_to_count(pool: PgPool) {
    let app = app(pool.clone()).await;
    let cookie = register(&app, "cageowner").await;

    // No lock: nothing to check.
    let res = post_json(&app, "/api/cage-check", &cookie, "{}").await;
    assert_eq!(res.status(), StatusCode::CONFLICT, "no cage, no check");

    let res = post_json(&app, "/api/lock", &cookie, "{}").await;
    assert_eq!(res.status(), StatusCode::CREATED);

    let res = post_json(&app, "/api/cage-check", &cookie, r#"{"note":"still on"}"#).await;
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = json_of(res).await;
    let check_id = body["id"].as_i64().expect("the check id");
    let seal = body["seal_code"].as_str().expect("a seal");
    assert_eq!(seal.len(), 8, "the seal is 8 characters: {seal}");
    assert_eq!(body["status"], "pending");
    assert!(
        body["claimed_secs"].as_i64().unwrap() >= 0,
        "claimed seconds are measured from the lock start"
    );

    // One a day.
    let res = post_json(&app, "/api/cage-check", &cookie, "{}").await;
    assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);

    // The owner cannot verify himself.
    let res = post_json(
        &app,
        &format!("/api/cage-check/{check_id}/attest"),
        &cookie,
        r#"{"verdict":"confirm"}"#,
    )
    .await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // Two peers at trust level 1.
    let mut peers = Vec::new();
    for name in ["cagepeer1", "cagepeer2"] {
        let c = register(&app, name).await;
        make_trust_1(&pool, name).await;
        peers.push(c);
    }

    let res = post_json(
        &app,
        &format!("/api/cage-check/{check_id}/attest"),
        &peers[0],
        r#"{"verdict":"confirm"}"#,
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json_of(res).await;
    assert_eq!(body["status"], "pending", "one voice is not a quorum");
    assert_eq!(body["confirms"], 1);

    let res = post_json(
        &app,
        &format!("/api/cage-check/{check_id}/attest"),
        &peers[1],
        r#"{"verdict":"confirm"}"#,
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json_of(res).await;
    assert_eq!(body["status"], "verified", "two independent voices verify");

    // And the metric the owner asked for now reflects it.
    let res = get(&app, "/api/cage", Some(&cookie)).await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json_of(res).await;
    assert!(
        body["metrics"]["verified_secs"].as_i64().unwrap() >= 0,
        "verified hours are reported"
    );
    assert_eq!(body["today"]["status"], "verified");
    assert!(
        body["total_verified_secs"].as_i64().is_some(),
        "the site-wide verified total is published"
    );
}

/// Confessions: public by default, and the setting is what takes them out.
#[sqlx::test(migrations = "./migrations")]
async fn the_feed_publishes_the_confession_until_the_author_sets_it_private(pool: PgPool) {
    let app = app(pool).await;
    let cookie = register(&app, "confessor").await;

    let res = post_json(
        &app,
        "/api/logs",
        &cookie,
        r#"{"kind":"habit","note":"I could not hold it"}"#,
    )
    .await;
    assert_eq!(res.status(), StatusCode::CREATED);

    let body = json_of(get(&app, "/api/feed", None).await).await;
    assert!(
        body["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["note"] == "I could not hold it"),
        "the confession is in the public feed by default"
    );

    let res = send(
        &app,
        "PATCH",
        "/api/profile",
        &cookie,
        Some(r#"{"logs_public":false}"#),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json_of(res).await;
    assert_eq!(body["logs_public"], false);

    let body = json_of(get(&app, "/api/feed", None).await).await;
    assert!(
        body["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|i| i["note"] != "I could not hold it"),
        "the setting takes it out of the public feed"
    );
}

/// Trust and the boards are readable, and the trust payload carries the progress
/// the page needs rather than a bare number.
#[sqlx::test(migrations = "./migrations")]
async fn trust_and_the_new_boards_are_readable(pool: PgPool) {
    let app = app(pool.clone()).await;
    let cookie = register(&app, "trustprobe").await;

    let res = get(&app, "/api/trust", Some(&cookie)).await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json_of(res).await;
    assert_eq!(body["level"], 0);
    assert_eq!(body["contributions"], 0);
    assert_eq!(body["is_leader"], false);

    let res = get(&app, "/api/metrics/boards", None).await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json_of(res).await;
    assert!(body["most_denied"].is_array());
    assert!(body["consistency"].is_array());
    assert!(body["locks"].is_array());

    let res = get(&app, "/api/metrics/time-to-cum", None).await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json_of(res).await;
    assert_eq!(body["aggregate"]["samples"], 0);
    assert!(body["mine"].is_null(), "an anonymous read has no personal block");
}

// ---------------------------------------------------------------------------
// 2026-10-29 (later) — the yesterday window, the export, the erasure.
// ---------------------------------------------------------------------------

/// The catch-up: legal once, for yesterday, only into an empty day, and judged by
/// yesterday's budget rather than today's.
#[sqlx::test(migrations = "./migrations")]
async fn a_catch_up_fills_yesterday_once_and_answers_to_its_own_day(pool: PgPool) {
    let app = app(pool.clone()).await;
    let cookie = register(&app, "latecomer").await;

    // The server tells the page whether the window is open; the page does not guess.
    let body = json_of(get(&app, "/api/stats", Some(&cookie)).await).await;
    assert_eq!(body["catch_up"]["available"], true);
    assert_eq!(body["catch_up"]["yesterday_count"], 0);

    let res = post_json(&app, "/api/logs", &cookie, r#"{"kind":"habit","day":"yesterday"}"#).await;
    assert_eq!(res.status(), StatusCode::CREATED, "the catch-up itself");
    let after = json_of(get(&app, "/api/stats", Some(&cookie)).await).await;
    assert_eq!(after["catch_up"]["available"], false, "one a day, and it is spent");
    assert_eq!(after["catch_up"]["filed_today"], true);
    assert_eq!(after["catch_up"]["yesterday_count"], 1);
    let body = json_of(res).await;
    let log = &body["stats"]["recent_logs"][0];
    assert_eq!(log["catch_up"], true, "the row is labelled: {log}");
    assert_eq!(
        body["stats"]["today_count"], 0,
        "it belongs to yesterday, so today's count stays 0"
    );
    assert_eq!(body["stats"]["current_streak"], 1, "and the streak is repaired");

    // Yesterday, in the database, is where it landed.
    let yesterday = time::OffsetDateTime::now_utc()
        .date()
        .previous_day()
        .unwrap();
    let (count,): (i64,) = sqlx::query_as(
        "select count(*) from habit_logs where log_date = $1 and is_catch_up",
    )
    .bind(yesterday)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 1);

    // The undo window follows the writing, not the day described.
    let id = log["id"].as_i64().unwrap();
    let res = send(
        &app,
        "PATCH",
        &format!("/api/logs/{id}"),
        &cookie,
        Some(r#"{"time_to_cum_secs":600}"#),
    )
    .await;
    assert_eq!(
        res.status(),
        StatusCode::OK,
        "a catch-up written a second ago is still inside its own 15 minutes"
    );

    // Once per filing day.
    let res = post_json(&app, "/api/logs", &cookie, r#"{"kind":"denial","day":"yesterday"}"#).await;
    assert_eq!(res.status(), StatusCode::CONFLICT);

    // A day that already has a log has no gap to fill.
    let other = register(&app, "alreadykept").await;
    let res = post_json(&app, "/api/logs", &other, r#"{"kind":"habit"}"#).await;
    assert_eq!(res.status(), StatusCode::CREATED);
    sqlx::query("update habit_logs set logged_at = now() - interval '1 day' where id = (select max(l.id) from habit_logs l join profiles p on p.id = l.user_id where p.username = 'alreadykept')")
        .execute(&pool)
        .await
        .unwrap();
    let res = post_json(&app, "/api/logs", &other, r#"{"kind":"habit","day":"yesterday"}"#).await;
    assert_eq!(
        res.status(),
        StatusCode::CONFLICT,
        "a catch-up fills a gap; it does not add to a day that was kept"
    );

    // Only ever yesterday, and only those two words.
    let res = post_json(&app, "/api/logs", &other, r#"{"kind":"habit","day":"2026-01-01"}"#).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // Yesterday's budget is what a catch-up spends: five wastes already there
    // means the sixth is refused, exactly as it would have been that day.
    let budgeted = register(&app, "budgetyesterday").await;
    let uid: uuid::Uuid =
        sqlx::query_scalar("select id from profiles where username = 'budgetyesterday'")
            .fetch_one(&pool)
            .await
            .unwrap();
    for _ in 0..5 {
        // Yesterday at 12:00 UTC, which is the day being tested — `now() - 1 day
        // + 12 hours` lands on TODAY at 08:00 and tests nothing.
        sqlx::query(
            "insert into habit_logs (user_id, kind, logged_at)
             values ($1, 'habit',
                     (((now() at time zone 'utc')::date - 1)::timestamp at time zone 'utc')
                     + interval '12 hours')",
        )
        .bind(uid)
        .execute(&pool)
        .await
        .unwrap();
    }
    // Five rows for yesterday means there is no gap; the day check fires first,
    // so prove the budget rule on a day with four rows and one space.
    sqlx::query("delete from habit_logs where id = (select max(id) from habit_logs where user_id = $1)")
        .bind(uid)
        .execute(&pool)
        .await
        .unwrap();
    let res = post_json(&app, "/api/logs", &budgeted, r#"{"kind":"habit","day":"yesterday"}"#).await;
    assert_eq!(
        res.status(),
        StatusCode::CONFLICT,
        "four rows is still a kept day: the gap rule is checked before the budget"
    );
}

/// The export: everything, only the caller's, and as a download.
#[sqlx::test(migrations = "./migrations")]
async fn the_export_holds_everything_and_only_your_own(pool: PgPool) {
    let app = app(pool.clone()).await;
    let cookie = register(&app, "exportee").await;
    let other = register(&app, "bystander").await;

    for c in [&cookie, &other] {
        let res = post_json(&app, "/api/logs", c, r#"{"kind":"habit","note":"mine"}"#).await;
        assert_eq!(res.status(), StatusCode::CREATED);
    }
    let res = post_json(&app, "/api/lock", &cookie, "{}").await;
    assert_eq!(res.status(), StatusCode::CREATED);
    let res = post_json(&app, "/api/cage-check", &cookie, "{}").await;
    assert_eq!(res.status(), StatusCode::CREATED);

    let res = get(&app, "/api/account/export", None).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED, "no session, no export");

    let res = get(&app, "/api/account/export", Some(&cookie)).await;
    assert_eq!(res.status(), StatusCode::OK);
    let disposition = res
        .headers()
        .get("content-disposition")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        disposition.starts_with("attachment; filename=\"streakforge-exportee-"),
        "the document arrives as a download: {disposition}"
    );
    let body = json_of(res).await;

    assert_eq!(body["format"], "streakforge.account.v1");
    assert_eq!(body["account"]["username"], "exportee");
    assert!(
        body["account"].get("password_hash").is_none(),
        "the hash is not part of anyone's data"
    );
    for section in [
        "logs",
        "locks",
        "cage_checks",
        "cage_attestations_given",
        "flags_filed",
        "moderation_actions_taken",
        "audit_events",
        "sessions",
    ] {
        assert!(
            body.get(section).is_some(),
            "the export is missing its `{section}` section"
        );
    }
    assert_eq!(body["logs"].as_array().unwrap().len(), 1);
    assert_eq!(body["locks"].as_array().unwrap().len(), 1);
    assert_eq!(body["cage_checks"].as_array().unwrap().len(), 1);
    assert_eq!(
        body["logs"][0]["note"], "mine",
        "the caller's own note travels with it"
    );
    assert!(
        !body.to_string().contains("bystander"),
        "another member's rows must not appear anywhere in the document"
    );
}

/// The erasure: password, a typed word, rows gone, one anonymous record left.
#[sqlx::test(migrations = "./migrations")]
async fn erasing_an_account_removes_its_data_and_keeps_only_an_anonymous_record(pool: PgPool) {
    let app = app(pool.clone()).await;
    let cookie = register(&app, "eraseme").await;
    let uid: uuid::Uuid = sqlx::query_scalar("select id from profiles where username = 'eraseme'")
        .fetch_one(&pool)
        .await
        .unwrap();

    post_json(&app, "/api/logs", &cookie, r#"{"kind":"habit","note":"gone"}"#).await;
    post_json(&app, "/api/lock", &cookie, "{}").await;
    let res = post_json(&app, "/api/cage-check", &cookie, "{}").await;
    let check_id = json_of(res).await["id"].as_i64().unwrap();
    let other: uuid::Uuid =
        sqlx::query_scalar("insert into profiles (username) values ('survivor') returning id")
            .fetch_one(&pool)
            .await
            .unwrap();
    // A decision this member took about someone else's content: it must survive
    // the erasure with a null actor, because it is a record about the content.
    sqlx::query(
        "insert into moderation_actions (actor_id, action, target_kind, target_id, note)
         values ($1, 'hide', 'cage_check', $2, 'test')",
    )
    .bind(uid)
    .bind(check_id)
    .execute(&pool)
    .await
    .unwrap();

    // Two refusals first.
    let res = post_json(&app, "/api/account/erase", &cookie, r#"{"password":"x","confirm":"yes"}"#).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST, "the word has to be typed");
    let res = post_json(&app, "/api/account/erase", &cookie, r#"{"password":"wrong-password","confirm":"DELETE"}"#).await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN, "the password is the proof");
    let still: i64 = sqlx::query_scalar("select count(*) from profiles where id = $1")
        .bind(uid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(still, 1, "a refused erasure erases nothing");

    let res = post_json(
        &app,
        "/api/account/erase",
        &cookie,
        r#"{"password":"correct-horse-battery","confirm":"DELETE"}"#,
    )
    .await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT, "with both, it goes");

    for (table, column) in [
        ("habit_logs", "user_id"),
        ("lock_sessions", "user_id"),
        ("cage_checks", "user_id"),
    ] {
        let left: i64 = sqlx::query_scalar(&format!(
            "select count(*) from {table} where {column} = $1"
        ))
        .bind(uid)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(left, 0, "{table} rows must be gone");
    }
    let profile: i64 = sqlx::query_scalar("select count(*) from profiles where id = $1")
        .bind(uid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(profile, 0);

    // The record that survives says what was erased, never who.
    let (detail, count): (serde_json::Value, i64) = sqlx::query_as(
        "select detail, count(*) over () from audit_log
          where action = 'account.erased' order by id desc limit 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 1, "one anonymous row");
    assert_eq!(detail["logs"], 1);
    assert!(
        !detail.to_string().contains("eraseme"),
        "the erasure record must not contain the erased username"
    );

    // The moderation decision survives, with no actor.
    let actor: Option<uuid::Uuid> =
        sqlx::query_scalar("select actor_id from moderation_actions order by id desc limit 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(actor.is_none(), "the action stays, the actor does not");

    // The session is gone too: the cookie no longer authenticates.
    let res = get(&app, "/api/auth/me", Some(&cookie)).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // And nothing else was touched: the bystander and their rows are intact.
    let survivors: i64 = sqlx::query_scalar("select count(*) from profiles where id = $1")
        .bind(other)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(survivors, 1);
}

// ---- the affirmation deck --------------------------------------------------

/// A member at trust level 2, built the way the ladder actually builds one: an account
/// older than the age gate, activity on three distinct days, five contributions. Using
/// SQL instead of a stored flag is the point — `is_leader` is the only stored bit, so a
/// test that "made someone a moderator" any other way would be testing a fiction.
async fn make_level_two(pool: &PgPool, username: &str) {
    sqlx::query("update profiles set created_at = now() - interval '3 days' where username = $1")
        .bind(username)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query(
        "insert into habit_logs (user_id, kind, note, logged_at)
         select p.id, 'habit', null,
                (((now() at time zone 'utc')::date - d)::timestamp at time zone 'utc') + interval '12 hours'
           from profiles p, generate_series(0, 2) d
          where p.username = $1",
    )
    .bind(username)
    .execute(pool)
    .await
    .unwrap();
    // Two more on the same day: contributions count logs, active days count dates.
    sqlx::query(
        "insert into habit_logs (user_id, kind, note, logged_at)
         select p.id, 'habit', null, now() - interval '10 minutes'
           from profiles p, generate_series(1, 2) where p.username = $1",
    )
    .bind(username)
    .execute(pool)
    .await
    .unwrap();
    let level: i32 = sqlx::query_scalar(
        "select level from public.trust_level((select id from profiles where username = $1))",
    )
    .bind(username)
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(level, 2, "the fixture must actually reach level 2");
}

async fn post(app: &Router, path: &str, body: serde_json::Value, cookie: Option<&str>) -> axum::response::Response {
    let mut req = Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json");
    if let Some(c) = cookie {
        req = req.header("cookie", c);
    }
    app.clone()
        .oneshot(req.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap()
}

async fn delete(app: &Router, path: &str, cookie: &str) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(path)
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn the_deck_is_readable_by_anyone_and_the_queue_is_not(pool: PgPool) {
    let app = app(pool).await;
    let res = get(&app, "/api/affirmations", None).await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json_of(res).await;
    assert!(
        body["deck"].as_array().unwrap().len() >= 18,
        "the curated deck is seeded: {}",
        body["deck"]
    );
    assert!(body["review"].as_array().unwrap().is_empty(), "no session, no queue");
    assert_eq!(body["can_review"], false);
    assert_eq!(body["remaining_today"], serde_json::Value::Null);
    // The seeded lines belong to nobody, and say so.
    assert_eq!(body["deck"][0]["author"], "curated");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_new_members_line_waits_for_a_level_two_voice(pool: PgPool) {
    let app = app(pool.clone()).await;
    let alice = register(&app, "deckalice").await;

    let res = post(
        &app,
        "/api/affirmations",
        serde_json::json!({ "body": "I am exactly where I put myself." }),
        Some(&alice),
    )
    .await;
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = json_of(res).await;
    assert_eq!(body["status"], "pending", "a level-0 line is a queue item");
    assert_eq!(body["live"], false);
    let id = body["id"].as_i64().unwrap();

    // Her own page shows it; the public deck does not.
    let mine = json_of(get(&app, "/api/affirmations", Some(&alice)).await).await;
    assert_eq!(mine["mine"].as_array().unwrap().len(), 1);
    assert_eq!(mine["can_review"], false);
    assert_eq!(mine["remaining_today"], 2);
    let public = json_of(get(&app, "/api/affirmations", None).await).await;
    assert!(
        !public["deck"].as_array().unwrap().iter().any(|i| i["id"] == id),
        "a pending line is not in the deck"
    );

    // A level-0 member cannot admit it.
    let bob = register(&app, "deckbob").await;
    let res = post(&app, &format!("/api/affirmations/{id}/approve"), serde_json::json!({}), Some(&bob)).await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN, "level 0 cannot admit");

    make_level_two(&pool, "deckbob").await;
    let res = post(&app, &format!("/api/affirmations/{id}/approve"), serde_json::json!({}), Some(&bob)).await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let public = json_of(get(&app, "/api/affirmations", None).await).await;
    let admitted = public["deck"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"] == id)
        .expect("the approved line is in the deck")
        .clone();
    assert_eq!(admitted["author"], "deckalice", "attributed to its author, not the approver");
    assert_eq!(admitted["status"], "live");

    // And it cannot be admitted twice: the second approval finds nothing pending.
    let res = post(&app, &format!("/api/affirmations/{id}/approve"), serde_json::json!({}), Some(&bob)).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_submission_is_one_line_and_a_paired_rate_limit(pool: PgPool) {
    let app = app(pool.clone()).await;
    let alice = register(&app, "deckcarol").await;

    let too_short = post(&app, "/api/affirmations", serde_json::json!({ "body": "   ab  " }), Some(&alice)).await;
    assert_eq!(too_short.status(), StatusCode::BAD_REQUEST, "three characters after tidying");

    let long = "x".repeat(201);
    let too_long = post(&app, "/api/affirmations", serde_json::json!({ "body": long }), Some(&alice)).await;
    assert_eq!(too_long.status(), StatusCode::BAD_REQUEST);

    // Pasted line breaks become one line, because the drill types it back.
    let res = post(
        &app,
        "/api/affirmations",
        serde_json::json!({ "body": "  I am calm.\n\n  I am  steady.  " }),
        Some(&alice),
    )
    .await;
    assert_eq!(res.status(), StatusCode::CREATED);
    assert_eq!(json_of(res).await["body"], "I am calm. I am steady.");

    // One an hour.
    let second = post(&app, "/api/affirmations", serde_json::json!({ "body": "Second line." }), Some(&alice)).await;
    assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(second.headers().get("retry-after").is_some(), "the 429 names the wait");

    // Age the two out of the hour window and spend the daily budget.
    sqlx::query("update affirmations set created_at = now() - interval '2 hours' where author_id = (select id from profiles where username = 'deckcarol')")
        .execute(&pool)
        .await
        .unwrap();
    let second = post(&app, "/api/affirmations", serde_json::json!({ "body": "Second line." }), Some(&alice)).await;
    assert_eq!(second.status(), StatusCode::CREATED, "an hour later is fine");
    sqlx::query("update affirmations set created_at = now() - interval '2 hours' where author_id = (select id from profiles where username = 'deckcarol')")
        .execute(&pool)
        .await
        .unwrap();
    let third = post(&app, "/api/affirmations", serde_json::json!({ "body": "Third line." }), Some(&alice)).await;
    assert_eq!(third.status(), StatusCode::CREATED);
    sqlx::query("update affirmations set created_at = now() - interval '2 hours' where author_id = (select id from profiles where username = 'deckcarol')")
        .execute(&pool)
        .await
        .unwrap();
    let fourth = post(&app, "/api/affirmations", serde_json::json!({ "body": "Fourth line." }), Some(&alice)).await;
    assert_eq!(fourth.status(), StatusCode::TOO_MANY_REQUESTS, "three a day");
    assert!(json_of(fourth).await["error"].as_str().unwrap().contains("3 a day"));
}

#[sqlx::test(migrations = "./migrations")]
async fn the_deck_keeps_one_copy_of_a_line(pool: PgPool) {
    let app = app(pool.clone()).await;
    register(&app, "deckdave").await;
    make_level_two(&pool, "deckdave").await;

    // A level-2 author goes straight into the deck...
    let cookie = login(&app, "deckdave").await;

    let seeded: String =
        sqlx::query_scalar("select body from affirmations where author_id is null limit 1")
            .fetch_one(&pool)
            .await
            .unwrap();

    let res = post(&app, "/api/affirmations", serde_json::json!({ "body": seeded }), Some(&cookie)).await;
    assert_eq!(res.status(), StatusCode::CONFLICT, "the curated line is already live");
    assert_eq!(json_of(res).await["status"], serde_json::Value::Null);

    // ...and the same words with different whitespace are still the same words.
    let spaced = format!("  {}  ", seeded.to_uppercase());
    let res = post(&app, "/api/affirmations", serde_json::json!({ "body": spaced }), Some(&cookie)).await;
    assert_eq!(res.status(), StatusCode::CONFLICT, "case and padding do not make a new line");
}

#[sqlx::test(migrations = "./migrations")]
async fn you_may_hide_and_delete_your_own_line_and_nobody_elses(pool: PgPool) {
    let app = app(pool.clone()).await;
    let alice = register(&app, "deckerin").await;
    let bob = register(&app, "deckfrank").await;

    let id = json_of(
        post(&app, "/api/affirmations", serde_json::json!({ "body": "Mine to keep." }), Some(&alice)).await,
    )
    .await["id"]
        .as_i64()
        .unwrap();

    // A stranger at the same level cannot touch it.
    let res = post(&app, &format!("/api/affirmations/{id}/hide"), serde_json::json!({}), Some(&bob)).await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    let res = delete(&app, &format!("/api/affirmations/{id}"), &bob).await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // The author can hide it — and it stays in her own list, marked.
    let res = post(&app, &format!("/api/affirmations/{id}/hide"), serde_json::json!({}), Some(&alice)).await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    let mine = json_of(get(&app, "/api/affirmations", Some(&alice)).await).await;
    assert_eq!(mine["mine"][0]["status"], "hidden", "a hidden line is still hers to see");
    let again = post(&app, &format!("/api/affirmations/{id}/hide"), serde_json::json!({}), Some(&alice)).await;
    assert_eq!(again.status(), StatusCode::CONFLICT, "hiding twice is not an error to swallow");

    // ...and delete it for good.
    let res = delete(&app, &format!("/api/affirmations/{id}"), &alice).await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    let mine = json_of(get(&app, "/api/affirmations", Some(&alice)).await).await;
    assert!(mine["mine"].as_array().unwrap().is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn a_curated_line_belongs_to_the_site(pool: PgPool) {
    let app = app(pool.clone()).await;
    let alice = register(&app, "deckgina").await;
    let id: i64 = sqlx::query_scalar("select id from affirmations where author_id is null limit 1")
        .fetch_one(&pool)
        .await
        .unwrap();

    let res = post(&app, &format!("/api/affirmations/{id}/hide"), serde_json::json!({}), Some(&alice)).await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN, "even a member may not hide curated copy");
    let res = delete(&app, &format!("/api/affirmations/{id}"), &alice).await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    sqlx::query("update profiles set is_leader = true where username = 'deckgina'")
        .execute(&pool)
        .await
        .unwrap();
    let res = delete(&app, &format!("/api/affirmations/{id}"), &alice).await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT, "the leader may prune the curated deck");
}

#[sqlx::test(migrations = "./migrations")]
async fn trust_ships_the_ladder_it_enforces(pool: PgPool) {
    let app = app(pool.clone()).await;
    let alice = register(&app, "ladderalice").await;

    let body = json_of(get(&app, "/api/trust", Some(&alice)).await).await;
    assert_eq!(body["level"], 0);
    assert_eq!(body["band"], "small", "a brand new site is on the soft band");
    assert_eq!(body["active_accounts"], 1, "floored at one, never zero");
    let rungs = body["thresholds"].as_array().unwrap();
    assert_eq!(rungs.len(), 3);
    assert_eq!(rungs[0]["level"], 1);
    assert_eq!(rungs[0]["min_contributions"], 1, "one log is enough for level 1");
    assert_eq!(rungs[1]["min_active_days"], 3, "level 2 is three days, not seven");
    assert_eq!(rungs[2]["min_contributions"], 15);

    // Ten active members later, the same member is judged against the grown ladder.
    make_level_two(&pool, "ladderalice").await;
    sqlx::query(
        "insert into profiles (username) select 'ladder' || g from generate_series(1, 10) g",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "insert into habit_logs (user_id, kind, note, logged_at)
         select p.id, 'habit', null, now() from profiles p where p.username like 'ladder%' and p.username <> 'ladderalice'",
    )
    .execute(&pool)
    .await
    .unwrap();

    let body = json_of(get(&app, "/api/trust", Some(&alice)).await).await;
    assert_eq!(body["band"], "grown");
    let rungs = body["thresholds"].as_array().unwrap();
    assert_eq!(rungs[1]["min_active_days"], 7, "the grown ladder is the one from 0016");
    assert_eq!(rungs[2]["min_contributions"], 40);
    assert_eq!(
        body["level"], 1,
        "three active days is level 2 on the soft band and level 1 on the grown one"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn time_to_cum_is_opt_in_and_the_switch_is_a_setting(pool: PgPool) {
    // 0021. The number this feature publishes is the most personal one the site holds, so the
    // default is the refusal: no consent, no pool. The test walks the actual toggle rather
    // than setting the column, because the column is not what a member touches.
    let app = app(pool.clone()).await;
    let alice = register(&app, "ttcalice").await;

    let anon = json_of(get(&app, "/api/metrics/time-to-cum", None).await).await;
    assert_eq!(anon["aggregate"]["reporters"], 0);
    assert_eq!(anon["sharing"], serde_json::Value::Null, "signed out has no setting");
    assert!(anon["board"].as_array().unwrap().is_empty());

    let mine = json_of(get(&app, "/api/metrics/time-to-cum", Some(&alice)).await).await;
    assert_eq!(mine["sharing"], false, "off is the default, and it says so");

    // She reports a duration; it is hers to see and nobody else's to average.
    post_json(&app, "/api/logs", &alice, r#"{"time_to_cum_secs":600}"#).await;
    let mine = json_of(get(&app, "/api/metrics/time-to-cum", Some(&alice)).await).await;
    assert_eq!(mine["mine"]["samples"], 1, "her own numbers, without sharing");
    assert_eq!(mine["aggregate"]["samples"], 0, "and not in the public pool");

    // The profile page carries the switch, and PATCH flips it.
    let profile = json_of(get(&app, "/api/profile/ttcalice", None).await).await;
    assert_eq!(profile["share_time_to_cum"], false);

    let res = send(&app, "PATCH", "/api/profile", &alice, Some(r#"{"share_time_to_cum":true}"#)).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(json_of(res).await["share_time_to_cum"], true);

    let after = json_of(get(&app, "/api/metrics/time-to-cum", None).await).await;
    assert_eq!(after["aggregate"]["samples"], 1, "her report is in the pool now");
    assert_eq!(after["aggregate"]["reporters"], 1);
    let mine = json_of(get(&app, "/api/metrics/time-to-cum", Some(&alice)).await).await;
    assert_eq!(mine["sharing"], true);

    // The numbers block on the profile: the average appears once she shares it, and the
    // trust level and verified cage time are there whether or not anyone shares anything.
    let profile = json_of(get(&app, "/api/profile/ttcalice", None).await).await;
    assert_eq!(profile["time_to_cum_avg_secs"], 600.0);
    assert_eq!(profile["time_to_cum_samples"], 1);
    assert_eq!(profile["trust_level"], 0, "a fresh account is level 0, and the profile says so");
    assert_eq!(profile["verified_cage_secs"], 0);

    // Turning it off is not a delete: the report stays, the publication stops.
    let res = send(&app, "PATCH", "/api/profile", &alice, Some(r#"{"share_time_to_cum":false}"#)).await;
    assert_eq!(res.status(), StatusCode::OK);
    let after = json_of(get(&app, "/api/metrics/time-to-cum", None).await).await;
    assert_eq!(after["aggregate"]["samples"], 0);
    let mine = json_of(get(&app, "/api/metrics/time-to-cum", Some(&alice)).await).await;
    assert_eq!(mine["mine"]["samples"], 1, "the row is still hers");

    // ...and the profile stops publishing the average, without losing the count: "not
    // shared" is a fact about four reports, not the same sentence as "no reports".
    let profile = json_of(get(&app, "/api/profile/ttcalice", None).await).await;
    assert_eq!(profile["time_to_cum_avg_secs"], serde_json::Value::Null);
    assert_eq!(profile["time_to_cum_samples"], 1);
}
