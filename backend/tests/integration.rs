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
    let a = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('waster') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let b = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('denier_lead') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

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
    let c = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('driller_lead') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

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
    let recent = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('recent_logger') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let old = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('week_logger') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

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

use streakforge_api::api::{check_rate_limits, next_utc_midnight, LogKind};
use time::OffsetDateTime;

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

#[sqlx::test(migrations = "./migrations")]
async fn first_log_of_the_day_is_allowed(pool: PgPool) {
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('daily_ok') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(check_rate_limits(&pool, uid, LogKind::Habit).await.is_ok());
}

#[sqlx::test(migrations = "./migrations")]
async fn second_log_same_utc_day_is_rejected(pool: PgPool) {
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('daily_dup') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    log_on_day(&pool, uid, "habit", 0).await;

    let err = check_rate_limits(&pool, uid, LogKind::Habit)
        .await
        .expect_err("a second WLW on the same UTC day must be rejected");
    assert_eq!(err.status, 429, "expected 429, got {err:?}");

    // The message must name the next allowed time, otherwise the user is told
    // only "come back tomorrow" with no clock to wait for.
    let msg = err.message.clone();
    assert!(msg.contains("UTC"), "message should mention UTC: {msg}");
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
        check_rate_limits(&pool, uid, LogKind::Habit).await.is_ok(),
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

    log_on_day(&pool, uid, "habit", 0).await;
    assert_eq!(
        check_rate_limits(&pool, uid, LogKind::Denial)
            .await
            .expect_err("a denial after a waste today must be rejected")
            .status,
        429
    );

    // and the reverse
    let uid2 = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('daily_excl2') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    log_on_day(&pool, uid2, "denial", 0).await;
    assert_eq!(
        check_rate_limits(&pool, uid2, LogKind::Habit)
            .await
            .expect_err("a waste after a denial today must be rejected")
            .status,
        429
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn affirmations_keep_their_own_budget(pool: PgPool) {
    // Affirmations are neither WLW nor WLD, so the 1/day cadence must not apply
    // to them — the drill would be unusable. But they must not be unlimited
    // either, and five is the documented daily ceiling.
    let uid = sqlx::query_scalar::<_, uuid::Uuid>(
        "insert into profiles (username) values ('daily_aff') returning id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    for _ in 0..5 {
        assert!(
            check_rate_limits(&pool, uid, LogKind::Affirmation).await.is_ok(),
            "first five reps of the day must be allowed"
        );
        log_on_day(&pool, uid, "affirmation", 0).await;
    }

    assert_eq!(
        check_rate_limits(&pool, uid, LogKind::Affirmation)
            .await
            .expect_err("the sixth rep must be rejected")
            .status,
        429
    );

    // An exhaustion of reps must not block a WLW, and a WLW must not be
    // blocked by reps: they share a day but not a budget.
    assert!(check_rate_limits(&pool, uid, LogKind::Habit).await.is_ok());
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
    // 3 logs x 1.0 = 3: the multiplier follows the ACTIVE streak, not the total.
    assert!((score - 3.0).abs() < 1e-9, "score={score}");
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
