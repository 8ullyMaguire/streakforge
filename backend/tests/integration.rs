// Backend integration tests — require a local Postgres (streakforge_test db).
// Run: DATABASE_URL=postgres://streakforge:***@127.0.0.1:5432/streakforge_test cargo test --test integration
//
// Auth flow tests exercise the SQL + auth logic directly (same code paths the
// HTTP handlers use). Bot-dissuasion checks (challenge, timing, honeypot) are
// covered as unit tests in auth.rs.

use sqlx::PgPool;
use streakforge_api::auth;
use streakforge_api::error::ApiError;

// These tests exercise the SQL functions directly against a test database.
// The HTTP layer is thin over these, so function-level coverage is the core.

#[sqlx::test(migrations = "./migrations")]
async fn streak_calculation_basic(pool: PgPool) {
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('tester') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // no logs -> streak 0
    let (cur, longest) = sqlx::query_as::<_, (i32, i32)>("select * from user_streak($1)")
        .bind(uid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!((cur, longest), (0, 0));

    // one log today -> streak 1
    sqlx::query("insert into habit_logs (user_id) values ($1)")
        .bind(uid)
        .execute(&pool)
        .await
        .unwrap();
    let (cur, longest) = sqlx::query_as::<_, (i32, i32)>("select * from user_streak($1)")
        .bind(uid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!((cur, longest), (1, 1));
}

#[sqlx::test(migrations = "./migrations")]
async fn streak_multiple_days(pool: PgPool) {
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('multi') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // 3 consecutive days: today-2, today-1, today
    for d in 0..3i32 {
        sqlx::query("insert into habit_logs (user_id, logged_at) values ($1, now() - ($2 || ' days')::interval)")
            .bind(uid)
            .bind(d)
            .execute(&pool)
            .await
            .unwrap();
    }
    let (cur, longest) = sqlx::query_as::<_, (i32, i32)>("select * from user_streak($1)")
        .bind(uid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!((cur, longest), (3, 3));

    // add a gap: 5 days ago (not consecutive) -> current stays 3, longest stays 3
    sqlx::query("insert into habit_logs (user_id, logged_at) values ($1, now() - interval '5 days')")
        .bind(uid)
        .execute(&pool)
        .await
        .unwrap();
    let (cur, longest) = sqlx::query_as::<_, (i32, i32)>("select * from user_streak($1)")
        .bind(uid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!((cur, longest), (3, 3));

    // add a longer run in the past: day 5 was inserted above, so 5..11 forms a 7-day run
    for d in [6i32, 7, 8, 9, 10, 11] {
        sqlx::query("insert into habit_logs (user_id, logged_at) values ($1, now() - ($2 || ' days')::interval)")
            .bind(uid)
            .bind(d)
            .execute(&pool)
            .await
            .unwrap();
    }
    let (cur, longest) = sqlx::query_as::<_, (i32, i32)>("select * from user_streak($1)")
        .bind(uid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(cur, 3); // current run ends today (3 days)
    assert_eq!(longest, 7); // 7-day run at days 5..11
}

#[sqlx::test(migrations = "./migrations")]
async fn leaderboard_ranking(pool: PgPool) {
    // two users, A logs 3 today, B logs 2 today
    let a = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('alpha') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let b = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('beta') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    for _ in 0..3 {
        sqlx::query("insert into habit_logs (user_id) values ($1)")
            .bind(a)
            .execute(&pool)
            .await
            .unwrap();
    }
    for _ in 0..2 {
        sqlx::query("insert into habit_logs (user_id) values ($1)")
            .bind(b)
            .execute(&pool)
            .await
            .unwrap();
    }
    let rows: Vec<(uuid::Uuid, String, i64)> =
        sqlx::query_as("select user_id, username, count from daily_leaderboard(10)")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].0, a);
    assert_eq!(rows[0].2, 3);
    assert_eq!(rows[1].0, b);
    assert_eq!(rows[1].2, 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn user_of_the_day_tiebreak(pool: PgPool) {
    // two users with same count today; earlier first log wins
    let a = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('early') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let b = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('late') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query("insert into habit_logs (user_id, logged_at) values ($1, now() - interval '2 hours')")
        .bind(a)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("insert into habit_logs (user_id, logged_at) values ($1, now() - interval '1 hour')")
        .bind(b)
        .execute(&pool)
        .await
        .unwrap();
    let row: (uuid::Uuid, String, i64) =
        sqlx::query_as("select user_id, username, count from user_of_the_day()")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(row.0, a); // early first log wins
    assert_eq!(row.1, "early");
    assert_eq!(row.2, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn feed_pagination(pool: PgPool) {
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('feeder') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    for _ in 0..5 {
        sqlx::query("insert into habit_logs (user_id) values ($1)")
            .bind(uid)
            .execute(&pool)
            .await
            .unwrap();
    }
    // page of 2
    let page1: Vec<(i64, uuid::Uuid, String)> =
        sqlx::query_as("select id, user_id, username from activity_feed(0, 2)")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(page1.len(), 2);
    let last_id = page1[1].0;
    // page 2 continues after last id
    let page2: Vec<(i64, uuid::Uuid, String)> =
        sqlx::query_as("select id, user_id, username from activity_feed($1, 2)")
            .bind(last_id)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(page2.len(), 2);
    assert!(page2[0].0 < last_id);
}

#[sqlx::test(migrations = "./migrations")]
async fn affirmation_kind_separated_from_community(pool: PgPool) {
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('driller') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // 2 habit logs + 3 affirmation reps
    for _ in 0..2 {
        sqlx::query("insert into habit_logs (user_id, kind) values ($1, 'habit')")
            .bind(uid)
            .execute(&pool)
            .await
            .unwrap();
    }
    for _ in 0..3 {
        sqlx::query("insert into habit_logs (user_id, kind) values ($1, 'affirmation')")
            .bind(uid)
            .execute(&pool)
            .await
            .unwrap();
    }

    // drill streak counts only affirmation days (today = 1 day -> streak 1)
    let (cur, longest): (i32, i32) =
        sqlx::query_as("select * from user_streak_kind($1, 'affirmation')")
            .bind(uid)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((cur, longest), (1, 1));

    // alltime leaderboard counts only habits
    let rows: Vec<(uuid::Uuid, i64)> =
        sqlx::query_as("select user_id, count from alltime_leaderboard(10)")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].1, 2); // 2 habits, not 5

    // feed excludes affirmations
    let feed: Vec<(i64, String)> =
        sqlx::query_as("select id, username from activity_feed(0, 10)")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(feed.len(), 2); // only habit logs

    // total excludes affirmations
    let total: i64 = sqlx::query_scalar("select * from total_logs()")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(total, 2);
}

// ---- Local auth (0004 migration) ----

/// Insert a profile exactly the way the register handler does (argon2 hash,
/// provider='local', provider_id NULL) and return its id.
async fn insert_local_user(pool: &PgPool, username: &str, password: &str) -> uuid::Uuid {
    let phc = auth::hash_password(password).expect("hash");
    sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username, display_name, avatar_url, provider, provider_id, password_hash)
         values ($1, NULL, NULL, 'local', NULL, $2)
         returning id",
    )
    .bind(username)
    .bind(&phc)
    .fetch_one(pool)
    .await
    .expect("insert local user")
}

/// Compute the challenge proof the way the client JS would (sha256 hex of
/// nonce || username || password, first 16 chars) — mirrored from auth.rs.
fn client_proof(nonce: &str, username: &str, password: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(nonce.as_bytes());
    hasher.update(username.as_bytes());
    hasher.update(password.as_bytes());
    let digest = hasher.finalize();
    digest[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()
}

#[sqlx::test(migrations = "./migrations")]
async fn register_creates_local_profile_with_argon2_hash(pool: PgPool) {
    let username = "new_forger";
    let password = "correct-horse-9";
    let phc = auth::hash_password(password).expect("hash");
    assert!(phc.starts_with("$argon2id$"), "PHC string: {phc}");

    let id: uuid::Uuid = sqlx::query_scalar(
        "insert into profiles (username, provider, provider_id, password_hash)
         values ($1, 'local', NULL, $2) returning id",
    )
    .bind(username)
    .bind(&phc)
    .fetch_one(&pool)
    .await
    .expect("insert");

    let (provider, provider_id, stored): (String, Option<String>, Option<String>) =
        sqlx::query_as("select provider, provider_id, password_hash from profiles where id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(provider, "local");
    assert_eq!(provider_id, None);
    let stored = stored.expect("local user has a hash");
    assert!(stored.starts_with("$argon2id$"));
    assert!(auth::verify_password(password, &stored));
    assert!(!auth::verify_password("wrong-password", &stored));
}

#[sqlx::test(migrations = "./migrations")]
async fn register_then_login_flow_with_social_url(pool: PgPool) {
    // register (as the handler would)
    let username = "flow_user";
    let password = "flow-password-123";
    let id = insert_local_user(&pool, username, password).await;

    // login: fetch by username + verify hash (handler code path)
    let row: (uuid::Uuid, Option<String>) =
        sqlx::query_as("select id, password_hash from profiles where username = $1")
            .bind(username)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(row.0, id);
    let phc = row.1.expect("local user has a hash");
    assert!(auth::verify_password(password, &phc));
    assert!(!auth::verify_password("wrong-password-1", &phc));

    // set social_url the way the PATCH handler does
    sqlx::query("update profiles set social_url = $1, updated_at = now() where id = $2")
        .bind("https://x.com/flow_user")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    // public profile route must return social_url
    let (uname, social): (String, Option<String>) =
        sqlx::query_as("select username, social_url from profiles where id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(uname, username);
    assert_eq!(social.as_deref(), Some("https://x.com/flow_user"));

    // fetch_profile helper (used to refresh the session user) round-trips it
    let session_user = auth::fetch_profile(&pool, id).await.expect("fetch_profile");
    assert_eq!(session_user.username, username);
    assert_eq!(session_user.provider, "local");
    assert_eq!(session_user.social_url.as_deref(), Some("https://x.com/flow_user"));
}

#[sqlx::test(migrations = "./migrations")]
async fn login_wrong_password_rejected(pool: PgPool) {
    let username = "wrongpw";
    insert_local_user(&pool, username, "right-password-1").await;

    let row: (uuid::Uuid, Option<String>) =
        sqlx::query_as("select id, password_hash from profiles where username = $1")
            .bind(username)
            .fetch_one(&pool)
            .await
            .unwrap();
    let phc = row.1.expect("hash");
    assert!(!auth::verify_password("wrong-password-1", &phc));
    // same generic message the login handler returns for any failure
    let err = ApiError::unauthorized("Invalid username or password");
    assert_eq!(err.status, axum::http::StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn legacy_provider_rows_survive_migration(pool: PgPool) {
    // rows inserted before 0004 (provider 'x'/'dev', no password_hash)
    sqlx::query("insert into profiles (username, provider, provider_id) values ('old_x', 'x', '12345')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("insert into profiles (username, provider, provider_id) values ('old_dev', 'dev', 'dev-old_dev')")
        .execute(&pool)
        .await
        .unwrap();

    let (provider, phc): (String, Option<String>) =
        sqlx::query_as("select provider, password_hash from profiles where username = 'old_x'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(provider, "x");
    assert_eq!(phc, None);

    // default provider for new rows is 'local'
    let (provider,): (String,) =
        sqlx::query_as("select provider from profiles where username = 'old_dev'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(provider, "dev");

    let (provider,): (String,) = sqlx::query_as(
        "insert into profiles (username) values ('new_default') returning provider",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(provider, "local");
}

#[sqlx::test(migrations = "./migrations")]
async fn duplicate_username_conflict(pool: PgPool) {
    let username = "dup_user";
    insert_local_user(&pool, username, "password-dup-1").await;

    // second insert with the same username -> unique violation
    let phc = auth::hash_password("password-dup-2").expect("hash");
    let err = sqlx::query(
        "insert into profiles (username, provider, provider_id, password_hash)
         values ($1, 'local', NULL, $2)",
    )
    .bind(username)
    .bind(&phc)
    .execute(&pool)
    .await
    .expect_err("duplicate username must violate the unique constraint");
    assert!(matches!(err, sqlx::Error::Database(ref db) if db.is_unique_violation()));

    // the ApiError mapping used by the register handler yields 409
    let api_err = ApiError::from(err);
    assert_eq!(api_err.status, axum::http::StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn social_url_validation_and_empty_clear(pool: PgPool) {
    let id = insert_local_user(&pool, "soc_user", "social-pass-1").await;

    // valid http(s) URL accepted
    sqlx::query("update profiles set social_url = $1 where id = $2")
        .bind("https://example.com/me")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    let (social,): (Option<String>,) =
        sqlx::query_as("select social_url from profiles where id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(social.as_deref(), Some("https://example.com/me"));

    // non-http URL rejected by the DB check constraint
    let err = sqlx::query("update profiles set social_url = $1 where id = $2")
        .bind("not-a-url")
        .bind(id)
        .execute(&pool)
        .await
        .expect_err("check constraint must reject non-http URLs");
    assert!(matches!(err, sqlx::Error::Database(ref db) if db.is_check_violation()));

    // empty string clears it (handler maps empty -> NULL before the update,
    // but the constraint also allows '' so this path stays safe)
    sqlx::query("update profiles set social_url = '' where id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    let (social,): (Option<String>,) =
        sqlx::query_as("select social_url from profiles where id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(social.as_deref(), Some(""));
}

#[sqlx::test(migrations = "./migrations")]
async fn challenge_proof_matches_client_computation(_pool: PgPool) {
    // The server recomputes what the client computed. Mirrors the JS contract:
    // proof = hex(sha256(nonce || username || password))[0..16]
    let nonce = auth::fresh_nonce();
    let username = "chal_user";
    let password = "challenge-pass-1";

    let server = auth::compute_challenge_proof(&nonce, username, password);
    let client = client_proof(&nonce, username, password);
    assert_eq!(server, client);
    assert_eq!(server.len(), 16);
    assert!(server.bytes().all(|b| b.is_ascii_hexdigit()));

    assert!(auth::challenge_proof_valid(&server, &nonce, username, password));
    // wrong proof (different password) fails
    let wrong = auth::compute_challenge_proof(&nonce, username, "wrong-password-1");
    assert!(!auth::challenge_proof_valid(&wrong, &nonce, username, password));
    // malformed proofs fail
    assert!(!auth::challenge_proof_valid("", &nonce, username, password));
    assert!(!auth::challenge_proof_valid("zzzzzzzzzzzzzzzz", &nonce, username, password));
}

// ---- Denial & lock subsystem (0005 migration) ----

#[sqlx::test(migrations = "./migrations")]
async fn denial_kind_separated_and_streak(pool: PgPool) {
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('denier') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // 2 denial logs today -> denial streak 1 day
    for _ in 0..2 {
        sqlx::query("insert into habit_logs (user_id, kind) values ($1, 'denial')")
            .bind(uid)
            .execute(&pool)
            .await
            .unwrap();
    }
    let (cur, longest): (i32, i32) =
        sqlx::query_as("select * from user_streak_kind($1, 'denial')")
            .bind(uid)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((cur, longest), (1, 1));

    // total_denied counts only denials
    let denied: i64 = sqlx::query_scalar("select * from total_denied()")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(denied, 2);

    // main total excludes denials
    let total: i64 = sqlx::query_scalar("select * from total_logs()")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(total, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn streak_beats_a_single_log_on_the_board(pool: PgPool) {
    // Under the Devotion Index a WLW and a WLD are worth the SAME (1 point each)
    // and are mutually exclusive, so neither outranks the other. What outranks
    // a single log is a STREAK: the multiplier. B logs 7 unbroken days, so
    // 7 x 1.25 = 8.75 and B leads A's single 1.0x log.
    //
    // (This test used to assert the old "denial = 10x waste" weighting, which
    // 0008 replaced. Mutually exclusive streaks make a 10x per-denial bonus
    // self-contradictory: you could never hold both.)
    // Aged fixtures: 0011 will not rank an account younger than 24 hours, and
    // this test is about the multiplier, not about the sybil floor.
    let a = insert_aged_user(&pool, "waster").await;
    let b = insert_aged_user(&pool, "denier_lead").await;

    sqlx::query("insert into habit_logs (user_id, kind) values ($1, 'habit')")
        .bind(a)
        .execute(&pool)
        .await
        .unwrap();
    for g in 0..7 {
        log_on_day(&pool, b, "denial", g).await;
    }

    let rows: Vec<(uuid::Uuid, f64, i64, i64, i64, f64)> = sqlx::query_as(
        "select user_id, score::float8, wlw::bigint, wld::bigint, game_bonus::bigint,
                multiplier::float8
         from monthly_leaderboard_weighted(10)",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].0, b, "the 7-day streak outranks a single log");
    assert!((rows[0].1 - 8.75).abs() < 1e-9, "B score={}", rows[0].1);
    assert_eq!(rows[0].2, 0, "B has no WLWs");
    assert_eq!(rows[0].3, 7, "B has seven WLDs");
    assert!((rows[0].5 - 1.25).abs() < 1e-9, "B multiplier={}", rows[0].5);

    assert_eq!(rows[1].0, a);
    assert!((rows[1].1 - 1.0).abs() < 1e-9, "A score={}", rows[1].1);
    assert_eq!(rows[1].2, 1, "A has one WLW");
    assert_eq!(rows[1].3, 0);
    assert_eq!(rows[1].4, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn three_affirmations_count_as_one_game_bonus(pool: PgPool) {
    // Affirmations are not WLWs or WLDs, so they never touch a streak. They
    // feed "Game Bonus" instead, at 1 point per 3 reps: 3 reps = 1, 6 = 2.
    // This is also why affirmations get their own daily budget — a strict
    // 1/day would make the 3-reps-per-point rule unreachable.
    // Aged for the same reason as above: the daily board has an age floor now.
    let c = insert_aged_user(&pool, "driller_lead").await;

    for _ in 0..3 {
        log_on_day(&pool, c, "affirmation", 0).await;
    }

    let (score, wlw, wld, game_bonus): (f64, i64, i64, i64) = sqlx::query_as(
        "select score::float8, wlw::bigint, wld::bigint, game_bonus::bigint
         from daily_leaderboard_weighted(10) where user_id = $1",
    )
    .bind(c)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(wlw, 0, "affirmations are not WLWs");
    assert_eq!(wld, 0, "affirmations are not WLDs");
    assert_eq!(game_bonus, 1, "3 affirmations = 1 game bonus");
    assert!((score - 1.0).abs() < 1e-9, "score={score}");
}

#[sqlx::test(migrations = "./migrations")]
async fn daily_uses_rolling_24h_window(pool: PgPool) {
    // A log 12 hours ago is NOT "today" UTC anymore, but MUST appear on the
    // daily board (rolling 24h). A log 3 days ago must NOT appear on daily
    // but SHOULD appear on weekly (rolling 7 days).
    // Aged: the window membership is what this test is about, and a profile
    // created this instant would be excluded by the 24-hour sybil floor.
    let recent = insert_aged_user(&pool, "recent_logger").await;
    let old = insert_aged_user(&pool, "week_logger").await;

    sqlx::query(
        "insert into habit_logs (user_id, kind, logged_at) values ($1, 'habit', now() - interval '12 hours')",
    )
    .bind(recent)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "insert into habit_logs (user_id, kind, logged_at) values ($1, 'habit', now() - interval '3 days')",
    )
    .bind(old)
    .execute(&pool)
    .await
    .unwrap();

    // The board column is `score` since 0008; these two tests only care about
    // window MEMBERSHIP, so they select just the id and leave the value alone.
    let daily_ids: Vec<uuid::Uuid> =
        sqlx::query_as::<_, (uuid::Uuid,)>("select user_id from daily_leaderboard_weighted(10)")
            .fetch_all(&pool)
            .await
            .unwrap()
            .into_iter()
            .map(|r| r.0)
            .collect();
    assert!(daily_ids.contains(&recent), "12h-old log must appear on daily");
    assert!(!daily_ids.contains(&old), "3d-old log must NOT appear on daily");

    let weekly_ids: Vec<uuid::Uuid> =
        sqlx::query_as::<_, (uuid::Uuid,)>("select user_id from weekly_leaderboard_weighted(10)")
            .fetch_all(&pool)
            .await
            .unwrap()
            .into_iter()
            .map(|r| r.0)
            .collect();
    assert!(weekly_ids.contains(&old), "3d-old log must appear on weekly");
    assert!(weekly_ids.contains(&recent), "12h-old log must appear on weekly");
}

#[sqlx::test(migrations = "./migrations")]
async fn lock_lifecycle_and_streak(pool: PgPool) {
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('caged') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // initially unlocked
    let (locked, cur, longest): (bool, i32, i32) =
        sqlx::query_as("select locked, current_streak, longest_streak from lock_info($1), lock_streak($1)")
            .bind(uid)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(!locked);
    assert_eq!((cur, longest), (0, 0));

    // lock now
    sqlx::query("insert into lock_sessions (user_id) values ($1)")
        .bind(uid)
        .execute(&pool)
        .await
        .unwrap();
    let (locked, cur, longest): (bool, i32, i32) =
        sqlx::query_as("select locked, current_streak, longest_streak from lock_info($1), lock_streak($1)")
            .bind(uid)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(locked);
    assert_eq!((cur, longest), (1, 1));

    // unlock with a reason, then re-lock shortly after (<=24h gap preserves streak)
    sqlx::query("update lock_sessions set unlocked_at = now(), reason = 'clean' where user_id = $1 and unlocked_at is null")
        .bind(uid)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("insert into lock_sessions (user_id) values ($1)")
        .bind(uid)
        .execute(&pool)
        .await
        .unwrap();
    let (locked, cur, longest): (bool, i32, i32) =
        sqlx::query_as("select locked, current_streak, longest_streak from lock_info($1), lock_streak($1)")
            .bind(uid)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(locked);
    assert_eq!(cur, 2); // 2 consecutive lock periods
    assert_eq!(longest, 2);

    // a lock >24h after the last unlock breaks the streak
    sqlx::query("update lock_sessions set unlocked_at = now() where user_id = $1 and unlocked_at is null")
        .bind(uid)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("insert into lock_sessions (user_id, locked_at) values ($1, now() + interval '25 hours')")
        .bind(uid)
        .execute(&pool)
        .await
        .unwrap();
    let (cur, longest): (i32, i32) =
        sqlx::query_as("select current_streak, longest_streak from lock_streak($1)")
            .bind(uid)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(cur, 1); // only the new lock period is current
    assert_eq!(longest, 2); // historical max preserved
}

#[sqlx::test(migrations = "./migrations")]
async fn lock_total_time_counts(pool: PgPool) {
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('longlock') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // 2-day lock that ended
    sqlx::query(
        "insert into lock_sessions (user_id, locked_at, unlocked_at) values ($1, now() - interval '2 days', now() - interval '1 day')",
    )
    .bind(uid)
    .execute(&pool)
    .await
    .unwrap();
    // current lock started 1 day ago
    sqlx::query("insert into lock_sessions (user_id, locked_at) values ($1, now() - interval '1 day')")
        .bind(uid)
        .execute(&pool)
        .await
        .unwrap();

    let (total_secs, longest_secs): (i64, i64) = sqlx::query_as(
        "select extract(epoch from total_locked)::bigint, extract(epoch from longest_lock)::bigint from lock_info($1)",
    )
    .bind(uid)
    .fetch_one(&pool)
    .await
    .unwrap();
    // 1 day closed + 1 day current = 2 days total; longest single = 1 day (both are ~1 day)
    assert!(total_secs >= 2 * 86400 - 120, "total_secs={total_secs}");
    assert!(longest_secs <= 1 * 86400 + 120, "longest_secs={longest_secs}");
}

// ---------------------------------------------------------------------------
// Daily (UTC) cadence — the rule that replaced the previous rolling-24h limit.
//
// These assert check_rate_limits directly. The HTTP layer is thin over it, and
// the failure mode this guards against is silent: with a 24h rolling window a
// user logging at 23:50 is locked until 23:50 next day, which reads as "the
// button is broken" rather than an error.
// ---------------------------------------------------------------------------

use streakforge_api::api::{check_rate_limits, daily_budget, next_utc_midnight, LogKind};
use time::OffsetDateTime;

/// How many logs of `kind` this user already has on the current UTC day.
async fn kind_today(pool: &PgPool, uid: uuid::Uuid, kind: &str) -> i64 {
    sqlx::query_scalar("select count(*) from habit_logs where user_id = $1 and kind = $2 and log_date = (now() at time zone 'utc')::date")
        .bind(uid)
        .bind(kind)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Insert a log for `uid` on the UTC day `days_ago` days back, at 12:00 UTC.
/// Noon keeps every case clear of the midnight boundary.
async fn log_on_day(pool: &PgPool, uid: uuid::Uuid, kind: &str, days_ago: i32) {
    // `kind` is a text column guarded by a CHECK constraint, not an enum type.
    sqlx::query(
        "insert into habit_logs (user_id, kind, logged_at)
         values ($1, $2,
                 (((now() at time zone 'utc')::date - ($3 || ' days')::interval
                   + interval '12 hours')::timestamptz))",
    )
    .bind(uid)
    .bind(kind)
    .bind(days_ago)
    .execute(pool)
    .await
    .unwrap();
}

/// The budget a user gets when the operator sets nothing: `WLW_DAILY_LIMIT`'s
/// default. Spelled out here so a change to the default has to change a test.
const WLW_TEST_LIMIT: i64 = 5;

#[sqlx::test(migrations = "./migrations")]
async fn first_log_of_the_day_is_allowed(pool: PgPool) {
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('daily_ok') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(
        check_rate_limits(&pool, uid, LogKind::Habit, WLW_TEST_LIMIT)
            .await
            .is_ok()
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn five_wastes_a_day_are_allowed_and_the_sixth_is_not(pool: PgPool) {
    // Loads wasted are a daily BUDGET, not the one-shot cadence they used to be.
    // The five allowed ones are the half a "is the sixth rejected" test cannot
    // see, so they are asserted one at a time.
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('wlw_budget') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    for n in 1..=5 {
        assert!(
            check_rate_limits(&pool, uid, LogKind::Habit, WLW_TEST_LIMIT)
                .await
                .is_ok(),
            "waste {n} of {WLW_TEST_LIMIT} must be allowed"
        );
        log_on_day(&pool, uid, "habit", 0).await;
    }

    let err = check_rate_limits(&pool, uid, LogKind::Habit, WLW_TEST_LIMIT)
        .await
        .expect_err("the sixth waste of the same UTC day must be rejected");
    assert_eq!(err.status, 429, "expected 429, got {err:?}");

    // The message must name both the budget and the clock. Without the time it
    // reads as "the button is broken"; without the number the user cannot tell
    // a budget from a bug.
    assert!(
        err.message.contains(&WLW_TEST_LIMIT.to_string()),
        "message should name the budget: {}",
        err.message
    );
    assert!(
        err.message.contains("UTC"),
        "message should mention UTC: {}",
        err.message
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn wlw_budget_is_the_configured_number(pool: PgPool) {
    // Configurability is the ask, so the limit must be an INPUT to enforcement.
    // 2 rather than the default 5, so a hardcoded 5 cannot pass this: with a
    // budget of 2 the third waste is already over, and the identical call with a
    // budget of 10 succeeds — proving the number is read, not stored.
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('wlw_config') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    for _ in 0..2 {
        assert!(check_rate_limits(&pool, uid, LogKind::Habit, 2).await.is_ok());
        log_on_day(&pool, uid, "habit", 0).await;
    }

    let err = check_rate_limits(&pool, uid, LogKind::Habit, 2)
        .await
        .expect_err("a third waste with WLW_DAILY_LIMIT=2 must be rejected");
    assert_eq!(err.status, 429);
    assert!(
        err.message.contains("2 loads wasted per UTC day"),
        "the message must quote the configured budget: {}",
        err.message
    );

    // Same user, same day, wider budget: allowed. The rule is the limit, not a
    // stored "has logged" flag.
    assert!(
        check_rate_limits(&pool, uid, LogKind::Habit, 10)
            .await
            .is_ok(),
        "a larger configured budget must still leave room"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn denial_stays_one_per_utc_day(pool: PgPool) {
    // Only wastes became a budget. A denial day is a stand, and the operator's
    // WLW_DAILY_LIMIT must not leak into it.
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('wld_cadence') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    log_on_day(&pool, uid, "denial", 0).await;

    let err = check_rate_limits(&pool, uid, LogKind::Denial, WLW_TEST_LIMIT)
        .await
        .expect_err("a second denial on the same UTC day must be rejected");
    assert_eq!(err.status, 429);
    assert!(
        err.message.contains("UTC"),
        "message should mention UTC: {}",
        err.message
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_log_yesterday_no_longer_blocks_today(pool: PgPool) {
    // This is the regression that the 24h rolling window failed: a log at 23:50
    // yesterday must not lock the user out until 23:50 today. The calendar-day
    // rule keys off log_date, so a prior-day log never blocks today.
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('daily_roll') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    log_on_day(&pool, uid, "habit", 1).await;

    assert!(
        check_rate_limits(&pool, uid, LogKind::Habit, WLW_TEST_LIMIT)
            .await
            .is_ok(),
        "yesterday's log must not block today"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn wlw_and_wld_are_mutually_exclusive(pool: PgPool) {
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('daily_excl') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // ONE waste spends the whole day, even though the waste budget (5) has four
    // left in it. Exclusivity is about the day, not about the budget.
    log_on_day(&pool, uid, "habit", 0).await;
    assert_eq!(
        check_rate_limits(&pool, uid, LogKind::Denial, WLW_TEST_LIMIT)
            .await
            .expect_err("a denial after a waste today must be rejected")
            .status,
        429
    );
    // ...and the waste budget itself is barely scratched by that one log.
    assert!(
        check_rate_limits(&pool, uid, LogKind::Habit, WLW_TEST_LIMIT)
            .await
            .is_ok(),
        "one waste must not exhaust a budget of five"
    );

    // and the reverse: a denial blocks every remaining waste slot
    let uid2 = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('daily_excl2') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    log_on_day(&pool, uid2, "denial", 0).await;
    assert_eq!(
        check_rate_limits(&pool, uid2, LogKind::Habit, WLW_TEST_LIMIT)
            .await
            .expect_err("a waste after a denial today must be rejected")
            .status,
        429
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn affirmations_keep_their_own_budget(pool: PgPool) {
    // Affirmations are neither WLW nor WLD, so neither the waste budget nor the
    // 1/day denial cadence applies to them — the drill would be unusable. But
    // they must not be unlimited either, and five is the documented ceiling.
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('daily_aff') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    for _ in 0..5 {
        assert!(
            check_rate_limits(&pool, uid, LogKind::Affirmation, WLW_TEST_LIMIT)
                .await
                .is_ok(),
            "first five reps of the day must be allowed"
        );
        log_on_day(&pool, uid, "affirmation", 0).await;
    }

    assert_eq!(
        check_rate_limits(&pool, uid, LogKind::Affirmation, WLW_TEST_LIMIT)
            .await
            .expect_err("the sixth rep must be rejected")
            .status,
        429
    );

    // An exhaustion of reps must not block a WLW, and a WLW must not be
    // blocked by reps: they share a day but not a budget.
    assert!(
        check_rate_limits(&pool, uid, LogKind::Habit, WLW_TEST_LIMIT)
            .await
            .is_ok()
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn enforcement_matches_the_reported_budget_for_every_kind(pool: PgPool) {
    // daily_budget() is what compute_stats tells the UI the limit is; the same
    // function decides what check_rate_limits enforces. They can only drift if
    // someone stops using the shared helper, so this walks all three kinds at a
    // NON-default budget and asserts the Nth submission is the last one allowed.
    for (kind, label) in [
        (LogKind::Habit, "habit"),
        (LogKind::Denial, "denial"),
        (LogKind::Affirmation, "affirmation"),
    ] {
        let budget = daily_budget(kind, 3);
        let uid = sqlx::query_scalar::<_, uuid::Uuid>(
            "insert into profiles (username) values ($1) returning id",
        )
        .bind(format!("budget_{label}"))
        .fetch_one(&pool)
        .await
        .unwrap();

        for n in 1..=budget {
            assert!(
                check_rate_limits(&pool, uid, kind, 3).await.is_ok(),
                "{label}: submission {n} of {budget} must be allowed"
            );
            log_on_day(&pool, uid, label, 0).await;
        }
        // Exactly `budget` submissions are in. The next one is the first that
        // is over, not the budget-th.
        assert_eq!(kind_today(&pool, uid, label).await, budget);
        assert_eq!(
            check_rate_limits(&pool, uid, kind, 3)
                .await
                .expect_err(&format!(
                    "{label}: submission {} must be rejected (budget {budget})",
                    budget + 1
                ))
                .status,
            429,
            "{label}: the budget the UI is told must be the budget enforced"
        );
    }
}

#[test]
fn next_midnight_is_always_tomorrow_midnight_utc() {
    let cases = [
        "2026-01-31T23:59:59Z",
        "2026-01-01T00:00:00Z",
        "2026-12-31T12:00:00Z",
    ];
    for c in cases {
        let now = OffsetDateTime::parse(c, &time::format_description::well_known::Rfc3339).unwrap();
        let next = next_utc_midnight(now);
        assert_eq!(next.date(), now.date() + time::Duration::days(1), "for {c}");
        assert_eq!(next.hour(), 0, "for {c}");
        assert_eq!(next.minute(), 0, "for {c}");
        assert_eq!(next.second(), 0, "for {c}");
        assert!(next > now, "next midnight must be in the future for {c}");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn devotion_index_uses_the_tier_multiplier(pool: PgPool) {
    // 7 unbroken UTC days of WLW -> streak 7 -> 1.25x -> 7 * 1.25 = 8.75.
    // g = 0 is today, so the chain is unbroken.
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('dev_tier') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    for g in 0..7 {
        log_on_day(&pool, uid, "habit", g).await;
    }

    let (wlw, wld, game_bonus, multiplier, score): (i64, i64, i64, f64, f64) = sqlx::query_as(
        "select wlw::bigint, wld::bigint, game_bonus::bigint,
                multiplier::float8, score::float8
         from devotion_index($1, now() - interval '30 days')",
    )
    .bind(uid)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(wlw, 7, "seven WLWs in the window");
    assert_eq!(wld, 0);
    assert_eq!(game_bonus, 0);
    assert!((multiplier - 1.25).abs() < 1e-9, "multiplier={multiplier}");
    assert!((score - 8.75).abs() < 1e-9, "score={score}");
}

#[sqlx::test(migrations = "./migrations")]
async fn devotion_index_streak_resets_after_a_missed_day(pool: PgPool) {
    // Days 0 and 1 are logged, day 2 is missed, day 3 is logged. The active
    // streak is 1 (today only), so the multiplier drops back to 1.0 even though
    // three logs sit in the window.
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('dev_reset') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    for g in [0, 1, 3] {
        log_on_day(&pool, uid, "habit", g).await;
    }

    let (wlw, multiplier, score): (i64, f64, f64) = sqlx::query_as(
        "select wlw::bigint, multiplier::float8, score::float8
         from devotion_index($1, now() - interval '30 days')",
    )
    .bind(uid)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(wlw, 3, "three WLWs in the window");
    assert!((multiplier - 1.0).abs() < 1e-9, "multiplier={multiplier}");
    // 3 logs x 1.0 = 3. Days 0 and 1 are consecutive, so presence is 2 days -- short
    // of the 7-day tier. Since 0012 the multiplier follows the PARTICIPATION streak
    // (every kind counts), so what costs it here is the missed day, not the total.
    assert!((score - 3.0).abs() < 1e-9, "score={score}");
}

#[sqlx::test(migrations = "./migrations")]
async fn multiplier_survives_alternating_waste_and_denial(pool: PgPool) {
    // The reported case: eight consecutive UTC days, alternating a waste and a
    // denial. The EXCLUSIVE streak resets on every change of kind (it is a run of
    // one thing); the MULTIPLIER must not, because she showed up every day.
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('dev_alt') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    for g in 0..8 {
        let kind = if g % 2 == 0 { "habit" } else { "denial" };
        log_on_day(&pool, uid, kind, g).await;
    }

    let (active_kind, streak, multiplier): (String, i64, f64) = sqlx::query_as(
        "select active_kind::text, streak::bigint, multiplier::float8
         from exclusive_streak($1)",
    )
    .bind(uid)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(
        active_kind == "habit" || active_kind == "denial",
        "active_kind={active_kind}"
    );
    assert_eq!(streak, 1, "the exclusive streak DOES reset on a kind change");
    assert!(
        (multiplier - 1.25).abs() < 1e-9,
        "eight days of showing up must earn 1.25 even when the kind alternates; got {multiplier}"
    );

    // And the number that actually scores: 4 WLW days + 4 WLD days = 8 x 1.25 = 10.
    let (wlw, wld, score, mul): (i64, i64, f64, f64) = sqlx::query_as(
        "select wlw::bigint, wld::bigint, score::float8, multiplier::float8
         from devotion_index($1, now() - interval '30 days')",
    )
    .bind(uid)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((wlw, wld), (4, 4), "four days of each kind");
    assert!((mul - 1.25).abs() < 1e-9, "scored multiplier={mul}");
    assert!((score - 10.0).abs() < 1e-9, "score={score}");

    // The new reading, directly.
    let (cur, longest): (i64, i64) =
        sqlx::query_as("select current_streak::bigint, longest_streak::bigint from user_participation_streak($1)")
            .bind(uid)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((cur, longest), (8, 8), "eight days of presence");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_drill_day_counts_as_a_day_of_participation(pool: PgPool) {
    // Eight days of waste with one drill (affirmation) in the middle. The drill ends
    // the exclusive streak's run -- affinities hold no streak -- but the day still
    // counts as showing up, so the multiplier keeps the 7-day tier.
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('dev_drill') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    for g in 0..8 {
        let kind = if g == 3 { "affirmation" } else { "habit" };
        log_on_day(&pool, uid, kind, g).await;
    }

    let (streak, multiplier): (i64, f64) = sqlx::query_as(
        "select streak::bigint, multiplier::float8 from exclusive_streak($1)",
    )
    .bind(uid)
    .fetch_one(&pool)
    .await
    .unwrap();

    // Today, yesterday and the day before are waste; the drill day interrupts it.
    assert_eq!(streak, 3, "the drill day interrupts the exclusive streak");
    assert!(
        (multiplier - 1.25).abs() < 1e-9,
        "a drill day still counts as participation; got {multiplier}"
    );

    // 7 WLW days x 1.25 = 8.75 (the drill is 1 rep: no bonus yet).
    let (wlw, game_bonus, score): (i64, i64, f64) = sqlx::query_as(
        "select wlw::bigint, game_bonus::bigint, score::float8
         from devotion_index($1, now() - interval '30 days')",
    )
    .bind(uid)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((wlw, game_bonus), (7, 0));
    assert!((score - 8.75).abs() < 1e-9, "score={score}");
}

#[sqlx::test(migrations = "./migrations")]
async fn participation_must_be_every_day_not_most_days(pool: PgPool) {
    // The boundary the change must NOT move: presence counts consecutive days, not
    // days-present. Three days ending today plus a five-day run further back is
    // eight days present and still 1.0x.
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('dev_gap') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    for g in [0, 1, 2, 5, 6, 7, 8, 9] {
        log_on_day(&pool, uid, "habit", g).await;
    }

    let (cur, longest): (i64, i64) =
        sqlx::query_as("select current_streak::bigint, longest_streak::bigint from user_participation_streak($1)")
            .bind(uid)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((cur, longest), (3, 5), "current run 3, longest run 5");

    let (wlw, multiplier, score): (i64, f64, f64) = sqlx::query_as(
        "select wlw::bigint, multiplier::float8, score::float8
         from devotion_index($1, now() - interval '30 days')",
    )
    .bind(uid)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(wlw, 8, "eight days present in the window");
    assert!(
        (multiplier - 1.0).abs() < 1e-9,
        "eight days present with a two-day gap is not a run; got {multiplier}"
    );
    assert!((score - 8.0).abs() < 1e-9, "score={score}");
}

#[sqlx::test(migrations = "./migrations")]
async fn monthly_leaderboard_is_the_default_period(pool: PgPool) {
    // The board defaults to monthly. With no period given, the handler must
    // apply the 30-day window — a log 10 days old is in scope, one 40 days old
    // is not.
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('lb_default') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    log_on_day(&pool, uid, "habit", 10).await;
    log_on_day(&pool, uid, "habit", 40).await;

    let (wlw_default,): (i64,) = sqlx::query_as(
        "select wlw::bigint from devotion_index($1, now() - interval '30 days')",
    )
    .bind(uid)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(wlw_default, 1, "only the 10-day-old log is inside the month");
}

// ---------------------------------------------------------------------------
// Score decay (0009): 30 days of inactivity wipes the score; lifetime survives.
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn score_survives_inside_the_30_day_window(pool: PgPool) {
    // Logs 5 and 29 days ago are both inside the 30-day window, so they count.
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('decay_live') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    log_on_day(&pool, uid, "habit", 5).await;
    log_on_day(&pool, uid, "habit", 29).await;

    let (wlw, score, lifetime, decayed): (i64, f64, i64, bool) = sqlx::query_as(
        "select wlw::bigint, score::float8, lifetime_total::bigint, decayed
         from devotion_index($1, null)",
    )
    .bind(uid)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(wlw, 2, "both logs are inside the window");
    assert!(score > 0.0, "score={score}");
    assert_eq!(lifetime, 2, "lifetime counts every log");
    assert!(!decayed, "an active user is not in the wiped state");
}

#[sqlx::test(migrations = "./migrations")]
async fn score_decays_to_zero_after_30_days_idle(pool: PgPool) {
    // The core rule: 30+ days of silence and the score is 0, while lifetime
    // keeps the record. This is the "no recovery" half — it stays 0 until a
    // new log lands.
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('decay_wiped') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // A month of daily logging, then a long silence.
    for g in (35..65).rev() {
        log_on_day(&pool, uid, "habit", g).await;
    }

    let (wlw, score, lifetime, decayed): (i64, f64, i64, bool) = sqlx::query_as(
        "select wlw::bigint, score::float8, lifetime_total::bigint, decayed
         from devotion_index($1, null)",
    )
    .bind(uid)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(wlw, 0, "no logs inside the window");
    assert_eq!(score, 0.0, "score is wiped after 30 idle days");
    assert_eq!(lifetime, 30, "lifetime still records all 30 logs");
    assert!(decayed, "user is in the wiped state");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_lapsed_user_drops_off_the_leaderboard(pool: PgPool) {
    // `where di.score > 0` must now also exclude the wiped, so a lapsed user
    // leaves the all-time board rather than sitting on it as a zero.
    // Aged, so that the first assertion is about live logs and not about the
    // 24-hour floor that would exclude a brand-new account anyway.
    let uid = insert_aged_user(&pool, "decay_board").await;

    // Inside the window at first, so the user genuinely appears on the board.
    for g in (1..5).rev() {
        log_on_day(&pool, uid, "habit", g).await;
    }
    assert!(
        sqlx::query_scalar::<_, bool>(
            "select exists (select 1 from alltime_leaderboard_weighted(50) where user_id = $1)",
        )
        .bind(uid)
        .fetch_one(&pool)
        .await
        .unwrap(),
        "must appear while the logs are still live"
    );

    // Age every log past the window.
    sqlx::query("update habit_logs set logged_at = logged_at - interval '60 days' where user_id = $1")
        .bind(uid)
        .execute(&pool)
        .await
        .unwrap();

    assert!(
        !sqlx::query_scalar::<_, bool>(
            "select exists (select 1 from alltime_leaderboard_weighted(50) where user_id = $1)",
        )
        .bind(uid)
        .fetch_one(&pool)
        .await
        .unwrap(),
        "a lapsed user must fall off the all-time board"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn logging_again_restores_the_score_from_lifetime(pool: PgPool) {
    // "No recovery" applies to the SCORE, not to the record: a new log starts a
    // fresh era, and lifetime proves the history was never actually lost.
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('decay_return') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    for g in (35..40).rev() {
        log_on_day(&pool, uid, "habit", g).await;
    }
    let (before,): (f64,) = sqlx::query_as("select score::float8 from devotion_index($1, null)")
        .bind(uid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(before, 0.0, "starts wiped");

    log_on_day(&pool, uid, "habit", 0).await;

    let (wlw, score, lifetime, decayed): (i64, f64, i64, bool) = sqlx::query_as(
        "select wlw::bigint, score::float8, lifetime_total::bigint, decayed
         from devotion_index($1, null)",
    )
    .bind(uid)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(wlw, 1, "only the new log counts: the 5 old ones are outside 30 days");
    assert!(score > 0.0, "a new log restores a nonzero score");
    assert_eq!(lifetime, 6, "lifetime spans the gap: 5 old + 1 new");
    assert!(!decayed, "no longer in the wiped state");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_brand_new_user_is_not_flagged_decayed(pool: PgPool) {
    // `decayed` must mean "had logs, lost them" — not "has no logs". A fresh
    // account is at 0 for a different reason and must not show the hint.
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('decay_new') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let (score, lifetime, decayed): (f64, i64, bool) = sqlx::query_as(
        "select score::float8, lifetime_total::bigint, decayed from devotion_index($1, null)",
    )
    .bind(uid)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(score, 0.0);
    assert_eq!(lifetime, 0);
    assert!(!decayed, "a user with no logs at all is not 'decayed'");
}

#[sqlx::test(migrations = "./migrations")]
async fn kpi_totals_and_trend(pool: PgPool) {
    let a = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username, created_at) values ('kpi_a', now() - interval '3 days') returning id",
    ).fetch_one(&pool).await.unwrap();
    let b = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username, created_at) values ('kpi_b', now() - interval '40 days') returning id",
    ).fetch_one(&pool).await.unwrap();
    // a: 2 wastes + 1 denial today-ish; 1 affirmation 2 days ago
    for _ in 0..2 {
        sqlx::query("insert into habit_logs (user_id, kind) values ($1, 'habit')")
            .bind(a).execute(&pool).await.unwrap();
    }
    sqlx::query("insert into habit_logs (user_id, kind) values ($1, 'denial')")
        .bind(a).execute(&pool).await.unwrap();
    sqlx::query("insert into habit_logs (user_id, kind, logged_at) values ($1, 'affirmation', now() - interval '2 days')")
        .bind(a).execute(&pool).await.unwrap();
    // b: 1 waste 30 hours ago (not in the 24h window, inside the trend)
    sqlx::query("insert into habit_logs (user_id, kind, logged_at) values ($1, 'habit', now() - interval '30 hours')")
        .bind(b).execute(&pool).await.unwrap();

    let (tw, td, ta, tu, active, new7, locked, lock_h, rate):
        (i64, i64, i64, i64, i64, i64, i64, f64, f64) =
        sqlx::query_as("select * from kpi_totals()").fetch_one(&pool).await.unwrap();
    assert_eq!(tw, 3, "2 today + 1 from 30h ago is 3 all-time");
    assert_eq!(td, 1);
    assert_eq!(ta, 1);
    assert_eq!(tu, 2);
    assert_eq!(active, 1, "only `a` logged within 24h; `b`'s log is 30h old");
    assert_eq!(new7, 1, "only `a` was created within 7 days");
    assert_eq!(locked, 0);
    assert!(lock_h == 0.0, "no lock sessions, so no hours; got {lock_h}");
    assert!((rate - 25.0).abs() < 0.1, "1 denied / 4 committed = 25%, got {rate}");

    // An open lock counts as currently_locked and accrues hours until now.
    sqlx::query("insert into lock_sessions (user_id, locked_at) values ($1, now() - interval '2 hours')")
        .bind(a).execute(&pool).await.unwrap();
    let (locked, lock_h): (i64, f64) =
        sqlx::query_as("select currently_locked, total_lock_hours from kpi_totals()")
            .fetch_one(&pool).await.unwrap();
    assert_eq!(locked, 1);
    assert!(lock_h >= 2.0, "an open 2h lock must count its elapsed hours; got {lock_h}");

    // The trend returns one row per day INCLUDING days with no activity, so a
    // chart does not silently compress its own x-axis.
    let trend: Vec<(String, i64, i64, i64)> =
        sqlx::query_as("select * from kpi_trend(30)").fetch_all(&pool).await.unwrap();
    assert_eq!(trend.len(), 30, "a 30-day trend must have 30 rows, empty days included");
    let today = trend.last().unwrap().clone();
    assert_eq!(today.1, 2, "wasted today");
    assert_eq!(today.2, 1, "denied today");
    let two_days_ago = trend[trend.len() - 3].clone();
    assert_eq!(two_days_ago.3, 1, "the affirmation 2 days ago");
}

#[sqlx::test(migrations = "./migrations")]
async fn kpi_totals_on_an_empty_site_are_zero_not_null(pool: PgPool) {
    // The page renders these on a fresh install. A NULL where a number belongs
    // is a 500 in the template, not a zero.
    let (tw, td, ta, tu, active, new7, locked, lock_h, rate):
        (i64, i64, i64, i64, i64, i64, i64, f64, f64) =
        sqlx::query_as("select * from kpi_totals()").fetch_one(&pool).await.unwrap();
    assert_eq!((tw, td, ta, tu, active, new7, locked), (0, 0, 0, 0, 0, 0, 0));
    assert!(lock_h == 0.0, "an empty site has no lock hours; got {lock_h}");
    assert!(rate == 0.0, "0 denied / 0 committed must be 0, not NaN; got {rate}");

    // And a shorter window is honoured, so the caller controls the span.
    let seven: Vec<(String, i64, i64, i64)> =
        sqlx::query_as("select * from kpi_trend(7)").fetch_all(&pool).await.unwrap();
    assert_eq!(seven.len(), 7);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_kpi_top_weekly_query_decodes(pool: PgPool) {
    // The plan called the /kpi handler a thin mapper needing no test, and for
    // the totals that is true. It is NOT true of the top-weekly board it reuses:
    // weekly_leaderboard_weighted returns score and multiplier as SQL numeric,
    // which sqlx cannot decode into f64. get_leaderboard already hit this and
    // works around it with an explicit column list plus a literal 0 for
    // racism_penalty. This pins that the KPI endpoint's copy of that query
    // decodes, because the failure otherwise appears only as a 500 in a
    // browser, on the one page whose whole job is to load.
    // Aged: the weekly board has a 24-hour account-age floor since 0011.
    let u = insert_aged_user(&pool, "kpi_board").await;
    sqlx::query("insert into habit_logs (user_id, kind) values ($1, 'habit')")
        .bind(u).execute(&pool).await.unwrap();
    sqlx::query("insert into habit_logs (user_id, kind) values ($1, 'denial')")
        .bind(u).execute(&pool).await.unwrap();

    type WeeklyRow = (
        uuid::Uuid, String, Option<String>, Option<String>,
        f64, i64, i64, i64, f64,
        i64, String, Option<OffsetDateTime>,
        f64,
        i64,
    );
    const WEEKLY_COLS: &str = "user_id, username, display_name, avatar_url, \
         score::float8, wlw::bigint, wld::bigint, game_bonus::bigint, \
         multiplier::float8, streak::bigint, active_kind, last_log_at, \
         0::float8, lifetime_total::bigint";
    let rows: Vec<WeeklyRow> = sqlx::query_as(&format!(
        "select {WEEKLY_COLS} from weekly_leaderboard_weighted(5)"
    )).fetch_all(&pool).await.unwrap();

    assert_eq!(rows.len(), 1, "the one active user should be on the weekly board");
    let r = &rows[0];
    assert_eq!(r.1, "kpi_board");
    assert!(r.4.is_finite(), "score decoded as a finite f64, got {}", r.4);
    assert!(r.8.is_finite(), "multiplier decoded as a finite f64, got {}", r.8);
    assert_eq!(r.12, 0.0, "racism_penalty is the literal 0, in its tuple position");
    assert_eq!(r.13, 2, "lifetime_total is the last column, so a shifted tuple would corrupt it");

    // The exact failure the explicit column list prevents: select * on a
    // numeric-returning function.
    let err = sqlx::query_as::<_, (uuid::Uuid, String, Option<String>, Option<String>,
        f64, i64, i64, i64, f64, i64, String, Option<OffsetDateTime>, f64, i64)>(
        "select * from weekly_leaderboard_weighted(5)")
        .fetch_all(&pool).await;
    assert!(err.is_err(), "select * must NOT decode here; if it now does, the casts in get_kpi are redundant and this test is asserting the wrong thing");
}

/// The mutations K1-K5 all live in the handler, above the SQL seam, so no
/// database-backed test can see them. These assert on the handler's source
/// instead -- the same technique this repo already uses for the kindred
/// manifest checks.
#[sqlx::test(migrations = "./migrations")]
async fn the_kpi_handler_casts_the_numeric_board_columns(pool: PgPool) {
    // Read at RUNTIME, not include_str!. include_str! embeds the file when the
    // test target is compiled, and a change to src/api.rs does not invalidate
    // that binary -- the test kept asserting against a stale copy and passed
    // with `rank: 999i64` sitting in the file on disk. A probe confirmed it
    // (rank999=false while the mutation was applied). include_str! is only
    // safe for a file that changes in the same commit as the test.
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/api.rs"),
    )
    .expect("src/api.rs is readable");

    let kpi = src.split("pub async fn get_kpi").nth(1)
        .expect("get_kpi exists");
    let kpi = &kpi[..kpi.find("\npub ").unwrap_or(kpi.len())];

    // K1: dropping `::float8` compiles fine and passes every SQL test, then 500s
    // on the first real request. get_leaderboard documents this exact trap.
    assert!(kpi.contains("score::float8"),
        "the KPI top-weekly query must cast score, or it 500s at runtime");
    assert!(kpi.contains("multiplier::float8"),
        "the KPI top-weekly query must cast multiplier for the same reason");

    // K2: the literal 0 stands in for racism_penalty, which the board function
    // does not return. It must sit before lifetime_total in the column list,
    // because the Row tuple is positional -- appended, it shifts every later
    // column and the handler decodes a bigint into an f64.
    //
    // Compared within the ONE column-list literal. The first version searched
    // the whole handler body, where the test's own const also appears, so the
    // comparison passed whether or not the mutation was applied.
    let cols = kpi
        .split("const WEEKLY_COLS")
        .nth(1)
        .and_then(|s| s.split("lifetime_total::bigint\";").next())
        .map(|s| format!("{s}lifetime_total::bigint\";"))
        .expect("the handler declares WEEKLY_COLS");
    let zero = cols.find("0::float8").expect("the literal 0 is in the column list");
    let lifetime = cols.find("lifetime_total::bigint").expect("lifetime_total cast is in the column list");
    assert!(zero < lifetime,
        "the racism_penalty literal must precede lifetime_total in the column list; after it, the tuple decodes shifted");

    // K3: the trend window is fixed at 30 days.
    assert!(kpi.contains("kpi_trend(30)"),
        "the trend window is 30 days; a shorter window silently shrinks the chart");
    // K4: rank is 1-based. Counted rather than merely present, because
    // get_leaderboard also assigns (i + 1).
    assert_eq!(kpi.matches("rank: (i + 1) as i64").count(), 1,
        "top_weekly must assign rank as (i + 1) exactly once");
    assert!(!kpi.contains("rank: i as i64"),
        "top_weekly ranks must start at 1, not 0");
    // And the endpoint is public: no auth extractor on the handler.
    assert!(kpi.contains("State(state): State<AppState>") && !kpi.contains("AuthUser"),
        "get_kpi must take no auth extractor -- the page is public by design");
    drop(pool);
}

// ===========================================================================
// 0011 — feed privacy, day-based scoring, sybil floor, decay countdown.
//
// The migration changes four behaviours at once; each one gets a test that
// fails if the rule is reversed, and the four public function signatures are
// pinned as literals (a changed OUT list decodes shifted or 500s at runtime,
// which no SQL test would otherwise catch).
// ===========================================================================

/// A profile that has existed for two days.
///
/// Needed because 0011 puts a 24-hour floor under leaderboard appearances, and
/// every leaderboard test here creates its user and logs in the same instant.
/// Backdating the FIXTURE keeps those tests about ranking; the floor itself is
/// asserted separately in `boards_wait_a_day_before_ranking_a_new_account`.
async fn insert_aged_user(pool: &PgPool, username: &str) -> uuid::Uuid {
    sqlx::query_scalar(
        "insert into profiles (username, created_at) values ($1, now() - interval '2 days')
         returning id",
    )
    .bind(username)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// One log at a given offset from now, in days (0 = today).
async fn log_at(pool: &PgPool, uid: uuid::Uuid, kind: &str, days_ago: i32) {
    sqlx::query(
        "insert into habit_logs (user_id, kind, logged_at)
         values ($1, $2, now() - ($3 || ' days')::interval)",
    )
    .bind(uid)
    .bind(kind)
    .bind(days_ago)
    .execute(pool)
    .await
    .unwrap();
}

/// The nine-value Devotion Index row, in the signature order 0011 defines.
type IndexRow = (i64, i64, i64, f64, f64, f64, i64, bool, Option<i64>);

async fn devotion(pool: &PgPool, uid: uuid::Uuid) -> IndexRow {
    sqlx::query_as(
        "select wlw::bigint, wld::bigint, game_bonus::bigint,
                multiplier::float8, racism_penalty::float8, score::float8,
                lifetime_total::bigint, decayed, days_until_decay::bigint
         from devotion_index($1, null)",
    )
    .bind(uid)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn feed_never_publishes_a_note(pool: PgPool) {
    let uid = insert_aged_user(&pool, "notekeeper").await;
    sqlx::query("insert into habit_logs (user_id, kind, note) values ($1, 'habit', 'private text')")
        .bind(uid)
        .execute(&pool)
        .await
        .unwrap();

    // The note is gone from the function's RESULT TYPE, not merely unselected.
    // A column the SQL cannot return is a column no handler can leak, and this
    // is the assertion that survives a future `select *`.
    let cols: String = sqlx::query_scalar(
        "select pg_get_function_result(p.oid)
         from pg_proc p join pg_namespace n on n.oid = p.pronamespace
         where n.nspname = 'public' and p.proname = 'activity_feed'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!cols.contains("note"),
        "activity_feed must not return a note column; a public feed of notes was the leak: {cols}");

    // And a row really does decode as the six columns the handler expects.
    // `logged_at::text` keeps this test free of a `time`/`chrono` type binding:
    // the point here is the column COUNT and order, not the timestamp type.
    let row = sqlx::query_as::<_, (i64, uuid::Uuid, String, Option<String>, Option<String>, String)>(
        "select id, user_id, username, display_name, avatar_url, logged_at::text
         from activity_feed(0, 50)",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.2, "notekeeper");
}

#[sqlx::test(migrations = "./migrations")]
async fn stats_still_hands_a_user_their_own_notes(_pool: PgPool) {
    // The dashboard reads notes back out of /api/stats; only the PUBLIC feed
    // lost them. This is a source check on the same principle as the KPI test:
    // the query that serves a user their own logs must still select `note`.
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/api.rs")).unwrap();
    let stats = src.split("async fn compute_stats").nth(1).expect("compute_stats exists");
    let stats = &stats[..stats.find("\nasync fn ").or(stats.find("\npub async fn ")).unwrap_or(stats.len())];
    assert!(stats.contains("note"),
        "compute_stats must still return the caller's own notes — the privacy change is about the public feed");
    // ...while the public feed handler must not use the note-carrying type.
    let feed = src.split("pub async fn get_feed").nth(1).expect("get_feed exists");
    let feed = &feed[..feed.find("\npub async fn ").unwrap_or(feed.len())];
    assert!(feed.contains("PublicFeedItem"), "get_feed must map into PublicFeedItem");
    assert!(!feed.contains("FeedItem {") || feed.contains("PublicFeedItem {"),
        "get_feed must not construct the note-carrying FeedItem");
}

#[sqlx::test(migrations = "./migrations")]
async fn scoring_counts_days_not_rows(pool: PgPool) {
    // Five logs on ONE UTC day.
    let farmer = insert_aged_user(&pool, "volume_farmer").await;
    for _ in 0..5 {
        log_at(&pool, farmer, "habit", 0).await;
    }
    let (wlw_farmer, _, _, _, _, score_farmer, lifetime_farmer, _, _) = devotion(&pool, farmer).await;
    assert_eq!(wlw_farmer, 1,
        "five logs on one day are one day of activity, not five: repetition must not buy rank");
    assert_eq!(lifetime_farmer, 5, "the lifetime record still counts rows — it is a record, not a score");

    // One log on each of five days.
    let steady = insert_aged_user(&pool, "steady").await;
    for d in 0..5 {
        log_at(&pool, steady, "habit", d).await;
    }
    let (wlw_steady, _, _, _, _, score_steady, _, _, _) = devotion(&pool, steady).await;
    assert_eq!(wlw_steady, 5, "five days of logging is five");
    assert_eq!(score_farmer, 1.0, "one day of wasting scores 1.0 before the multiplier");
    assert!(score_steady > score_farmer,
        "consistency must beat volume: {score_steady} vs {score_farmer}");
}

#[sqlx::test(migrations = "./migrations")]
async fn denials_also_count_by_day(pool: PgPool) {
    let uid = insert_aged_user(&pool, "denier").await;
    for _ in 0..4 {
        log_at(&pool, uid, "denial", 0).await;
    }
    let (_, wld, _, _, _, _, _, _, _) = devotion(&pool, uid).await;
    assert_eq!(wld, 1, "a day of denial counts once, like a day of waste");
}

#[sqlx::test(migrations = "./migrations")]
async fn boards_wait_a_day_before_ranking_a_new_account(pool: PgPool) {
    // A brand-new account with a log today.
    // Created moments ago, which is the whole point: this account must be
    // refused a board slot.
    let fresh: uuid::Uuid = sqlx::query_scalar(
        "insert into profiles (username) values ('brand_new') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    log_at(&pool, fresh, "habit", 0).await;

    let ranked: i64 = sqlx::query_scalar("select count(*) from daily_leaderboard_weighted(50)")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(ranked, 0, "an account younger than 24 hours must not hold a board slot");

    // The same account, one day older, ranks.
    sqlx::query("update profiles set created_at = now() - interval '25 hours' where id = $1")
        .bind(fresh)
        .execute(&pool)
        .await
        .unwrap();
    let ranked: i64 = sqlx::query_scalar("select count(*) from daily_leaderboard_weighted(50)")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(ranked, 1, "the rule is an age floor, not a ban");
}

#[sqlx::test(migrations = "./migrations")]
async fn decay_countdown_is_reported_on_the_index(pool: PgPool) {
    // Never logged: nothing to lose, so no countdown rather than a false "30".
    let quiet = insert_aged_user(&pool, "never_logged").await;
    let (_, _, _, _, _, _, _, decayed, days) = devotion(&pool, quiet).await;
    assert_eq!(days, None, "a user with no logs has no decay countdown");
    assert!(!decayed);

    // A log 25 days old: five days of grace left (30 - 25).
    let twentyfive = insert_aged_user(&pool, "nearly_gone").await;
    log_at(&pool, twentyfive, "habit", 25).await;
    let (_, _, _, _, _, _, _, decayed, days) = devotion(&pool, twentyfive).await;
    assert_eq!(days, Some(5), "25 days of silence leaves 5 of the 30-day era");
    assert!(!decayed);

    // A log 40 days old: the era already ended, and the countdown says 0 rather
    // than going negative, so the UI can distinguish "nearly gone" from "gone".
    let forty = insert_aged_user(&pool, "long_gone").await;
    log_at(&pool, forty, "habit", 40).await;
    let (_, _, _, _, _, score, _, decayed, days) = devotion(&pool, forty).await;
    assert_eq!(days, Some(0));
    assert!(decayed, "logs outside the horizon must report as decayed");
    assert_eq!(score, 0.0, "a decayed score is zero, which is why the warning matters");
}

#[sqlx::test(migrations = "./migrations")]
async fn today_counts_only_counts_today(pool: PgPool) {
    let a = insert_aged_user(&pool, "today_a").await;
    let b = insert_aged_user(&pool, "today_b").await;
    for _ in 0..3 {
        log_at(&pool, a, "denial", 0).await;
    }
    log_at(&pool, a, "habit", 0).await;
    log_at(&pool, b, "habit", 0).await;
    // Two days ago: must not be in "today" by any of the three counters.
    log_at(&pool, b, "habit", 2).await;
    log_at(&pool, b, "denial", 2).await;

    let (denials, wastes, loggers): (i64, i64, i64) =
        sqlx::query_as("select denials_today::bigint, wastes_today::bigint, loggers_today::bigint from today_counts()")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(denials, 3, "three denials were logged today");
    assert_eq!(wastes, 2, "two wastes were logged today");
    assert_eq!(loggers, 2, "two distinct people logged something today");
}

#[sqlx::test(migrations = "./migrations")]
async fn public_function_signatures_are_pinned(pool: PgPool) {
    // Exact literal comparison, deliberately brittle: SQLx decodes these rows
    // positionally, so a renamed, retyped or reordered column is a runtime 500
    // on a public page with every test still green. Adding a column here is
    // meant to be a deliberate act that starts with editing this list.
    let expected: [(&str, &str); 5] = [
        (
            "activity_feed",
            "TABLE(id bigint, user_id uuid, username text, display_name text, avatar_url text, logged_at timestamp with time zone)",
        ),
        (
            "devotion_index",
            "TABLE(wlw bigint, wld bigint, game_bonus bigint, multiplier numeric, racism_penalty numeric, score numeric, lifetime_total bigint, decayed boolean, days_until_decay bigint)",
        ),
        (
            "devotion_leaderboard",
            "TABLE(user_id uuid, username text, display_name text, avatar_url text, score numeric, wlw bigint, wld bigint, game_bonus bigint, multiplier numeric, streak integer, active_kind text, last_log_at timestamp with time zone, lifetime_total bigint)",
        ),
        (
            "kpi_totals",
            "TABLE(total_wasted bigint, total_denied bigint, total_affirmations bigint, total_users bigint, active_24h bigint, new_7d bigint, currently_locked bigint, total_lock_hours double precision, denial_rate double precision)",
        ),
        (
            "today_counts",
            "TABLE(denials_today bigint, wastes_today bigint, loggers_today bigint)",
        ),
    ];
    for (name, want) in expected {
        let got: String = sqlx::query_scalar(
            "select regexp_replace(pg_get_function_result(p.oid), '\\s+', ' ', 'g')
             from pg_proc p join pg_namespace n on n.oid = p.pronamespace
             where n.nspname = 'public' and p.proname = $1",
        )
        .bind(name)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(got, want, "the column list of {name}() changed; the Rust tuple must change with it");
    }
}
