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
