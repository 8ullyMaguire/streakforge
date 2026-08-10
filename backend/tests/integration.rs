// Backend integration tests — require a local Postgres (streakforge_test db).
// Run: DATABASE_URL=postgres://streakforge:streakforge_dev@127.0.0.1:5432/streakforge_test cargo test

use sqlx::PgPool;
use streakforge_api::*; // (will expose helpers via lib.rs)

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
