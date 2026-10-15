//! The streak engine's edge-case table: timezones, DST and the grace window.
//!
//! The 2026-10-04 ranking's verification ask for this repo was literally "add an
//! edge-case table for streak calculation (timezones, DST, grace periods)". The
//! engine's day semantics live in a generated column, three near-identical SQL
//! scanners that a comment asks the next reader to "keep in step", and one
//! migration whose measured claims ("must be 1.25") nothing pinned. This file is
//! that table, and it is where those claims become assertions.
//!
//! Run: DATABASE_URL=postgres://postgres:postgres@127.0.0.1:55433/postgres \
//!        cargo test --test streak_edge_cases
//!
//! Every expected value below is a claim taken from the code, not invented here:
//!
//! * `0001` — `log_date` is `generated always as ((logged_at at time zone 'utc')::date)`,
//!   and the current streak is the run ending today "or yesterday if today is empty".
//! * `0005` — the *lock* streak allows a 24h unlock gap. The day streaks have no
//!   wider window than the today-or-yesterday rule.
//! * `0008` — `devotion_multiplier`'s tiers (1.0 / 1.25 at 7 / 1.5 at 14 / 2.0 at 30).
//! * `0012` — the multiplier reads DAILY PARTICIPATION (any kind) while the
//!   exclusive streak still resets on a kind change, with measured rows.
//! * `api.rs` — "Matches user_streak_kind (UTC-day streaks) and the Devotion Index
//!   multiplier": the Rust day and the SQL day must agree.

use sqlx::PgPool;

async fn new_user(pool: &PgPool, name: &str) -> uuid::Uuid {
    sqlx::query_scalar::<_, uuid::Uuid>("insert into profiles (username) values ($1) returning id")
        .bind(name)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Insert a log at NOON UTC, `offset` days from today (UTC).
///
/// Noon, not "now": with a fixed time-of-day the row's day is decided by the
/// offset alone, so this table cannot go flaky when it runs near a boundary.
async fn log_on(pool: &PgPool, uid: uuid::Uuid, kind: &str, offset: i32) {
    sqlx::query(
        "insert into habit_logs (user_id, kind, logged_at)
         values ($1, $2, (((now() at time zone 'utc')::date + $3) + time '12:00') at time zone 'utc')",
    )
    .bind(uid)
    .bind(kind)
    .bind(offset)
    .execute(pool)
    .await
    .unwrap();
}

async fn streak(pool: &PgPool, uid: uuid::Uuid) -> (i32, i32) {
    sqlx::query_as("select * from user_streak($1)")
        .bind(uid)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn participation(pool: &PgPool, uid: uuid::Uuid) -> (i32, i32) {
    sqlx::query_as("select * from user_participation_streak($1)")
        .bind(uid)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// The one function that returns both halves of 0012: the kind run (which may
/// reset when the member changes kind) and the platform's ONE multiplier (read
/// off the participation streak). `::float8` because `numeric` is awkward here and
/// this is the same cast the existing board test uses.
async fn exclusive(pool: &PgPool, uid: uuid::Uuid) -> (String, i32, f64) {
    sqlx::query_as("select active_kind, streak, multiplier::float8 from exclusive_streak($1)")
        .bind(uid)
        .fetch_one(pool)
        .await
        .unwrap()
}

// ---------------------------------------------------------------------------
// 1. The day boundary, as a table of instants
// ---------------------------------------------------------------------------
//
// The engine's day does not follow the user's clock: it is UTC, always, and the
// rule lives in the schema as a generated column so no caller can drift from it.
// These are the rows a local-day implementation would get wrong.

#[sqlx::test(migrations = "./migrations")]
async fn the_utc_day_is_the_logged_instant_converted_to_utc(pool: PgPool) {
    let cases = [
        ("2026-07-15 12:00:00+00", "2026-07-15", "plain UTC midday"),
        (
            "2026-07-14 23:59:59+00",
            "2026-07-14",
            "one second before the flip",
        ),
        (
            "2026-07-15 00:00:00+00",
            "2026-07-15",
            "the flip instant itself, with no grace on either side",
        ),
        // The owner's own zone, +02 in summer: 01:30 local is 23:30 the PREVIOUS
        // UTC day, so a log just after local midnight is filed under yesterday.
        (
            "2026-07-15 01:30:00+02",
            "2026-07-14",
            "01:30 CEST is filed under the previous UTC day",
        ),
        // The same local clock time in winter lands on a DIFFERENT UTC day,
        // because +01 is inside the boundary and +02 is outside it.
        (
            "2026-01-15 01:30:00+01",
            "2026-01-15",
            "01:30 CET is filed under the same UTC day",
        ),
        // DST, spring forward: Europe/Berlin 2026-03-29 02:00 CET -> 03:00 CEST.
        ("2026-03-29 01:59:59+01", "2026-03-29", "last second of CET"),
        (
            "2026-03-29 03:00:01+02",
            "2026-03-29",
            "first second of CEST: one local hour later, same UTC day",
        ),
        // DST, fall back: 2026-10-25 03:00 CEST -> 02:00 CET, so 02:00-02:59 local
        // happens twice. The repeated hour is 00:00-01:59 UTC, so it can never
        // straddle a UTC day -- the two passes agree.
        (
            "2026-10-25 02:30:00+02",
            "2026-10-25",
            "first pass of the repeated local hour",
        ),
        (
            "2026-10-25 02:30:00+01",
            "2026-10-25",
            "second pass of the same local hour: still the same UTC day",
        ),
    ];

    for (instant, want, why) in cases {
        let got: String =
            sqlx::query_scalar("select (($1::timestamptz) at time zone 'utc')::date::text")
                .bind(instant)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(got, want, "{instant} must be filed under {want} ({why})");
    }

    // The rule is the SCHEMA's, not a caller's: the stored column must derive the
    // same day for the same instant.
    let uid = new_user(&pool, "tz_column").await;
    sqlx::query(
        "insert into habit_logs (user_id, kind, logged_at)
         values ($1, 'habit', timestamptz '2026-07-15 01:30:00+02')",
    )
    .bind(uid)
    .execute(&pool)
    .await
    .unwrap();
    let filed: String =
        sqlx::query_scalar("select log_date::text from habit_logs where user_id = $1")
            .bind(uid)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        filed, "2026-07-14",
        "the generated column must apply the same rule the scanners read"
    );
}

/// The fixture that distinguishes a UTC day from a local one.
///
/// Two logs on the SAME UTC day, three hours apart, at 00:30 and 23:30 UTC. For a
/// +02 member those are 02:30 and 01:30 *the next local day*, so an engine that
/// bucketed by local date would count two days. Asserting one day here is what
/// makes the timezone property testable at all -- a table of instants alone would
/// still pass if the scanners started bucketing locally.
#[sqlx::test(migrations = "./migrations")]
async fn one_utc_day_is_one_day_however_the_local_clock_reads(pool: PgPool) {
    let uid = new_user(&pool, "two_ends").await;
    sqlx::query(
        "insert into habit_logs (user_id, kind, logged_at)
         values ($1, 'habit', ((now() at time zone 'utc')::date + time '00:30') at time zone 'utc'),
                ($1, 'habit', ((now() at time zone 'utc')::date + time '23:30') at time zone 'utc')",
    )
    .bind(uid)
    .execute(&pool)
    .await
    .unwrap();

    let days: i64 =
        sqlx::query_scalar("select count(distinct log_date) from habit_logs where user_id = $1")
            .bind(uid)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(days, 1, "00:30 and 23:30 UTC are one day");
    assert_eq!(
        participation(&pool, uid).await,
        (1, 1),
        "and the streak agrees with the stored day"
    );
}

// ---------------------------------------------------------------------------
// 2. The grace window
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn the_grace_window_is_exactly_one_utc_day(pool: PgPool) {
    // The window is spelled out SEPARATELY in 0001 (`user_streak`) and 0012
    // (`user_participation_streak`), so each case below asserts BOTH scanners --
    // otherwise a mutation to either one's window survives on the other's fixture.

    // A run ending TODAY is the current streak.
    let a = new_user(&pool, "grace_today").await;
    for off in [-2, -1, 0] {
        log_on(&pool, a, "habit", off).await;
    }
    assert_eq!(
        streak(&pool, a).await,
        (3, 3),
        "a run ending today is current"
    );
    assert_eq!(
        participation(&pool, a).await,
        (3, 3),
        "and so is the participation run"
    );

    // A run ending YESTERDAY still counts. That is the whole grace window, and it
    // is why the engine needs no "grace period" setting: you have until the end of
    // today to continue yesterday's run.
    let b = new_user(&pool, "grace_yest").await;
    for off in [-3, -2, -1] {
        log_on(&pool, b, "habit", off).await;
    }
    assert_eq!(
        streak(&pool, b).await,
        (3, 3),
        "a run ending yesterday is still live"
    );
    assert_eq!(participation(&pool, b).await, (3, 3), "for both scanners");

    // Today extends yesterday's run rather than starting a new one.
    let c = new_user(&pool, "grace_extend").await;
    for off in [-3, -2, -1] {
        log_on(&pool, c, "habit", off).await;
    }
    log_on(&pool, c, "habit", 0).await;
    assert_eq!(
        streak(&pool, c).await,
        (4, 4),
        "today continues the run, it does not restart it"
    );
    assert_eq!(participation(&pool, c).await, (4, 4), "for both scanners");

    // One more empty day and the run is over -- but `longest` keeps it.
    let d = new_user(&pool, "grace_broken").await;
    for off in [-4, -3, -2] {
        log_on(&pool, d, "habit", off).await;
    }
    assert_eq!(
        streak(&pool, d).await,
        (0, 3),
        "a run that ended two days ago is not current"
    );
    assert_eq!(participation(&pool, d).await, (0, 3), "for both scanners");

    // There is no grace on the other side either: the boundary is UTC midnight and
    // a log one second after it belongs to the new day (see the table above).
    // A FUTURE-dated log -- clock skew, or a direct write -- is counted by
    // `longest` and cannot be `current`, because the newest island does not end
    // today or yesterday. Pinned as the behaviour it is, not as one to rely on.
    let e = new_user(&pool, "grace_future").await;
    for off in [-1, 0, 1] {
        log_on(&pool, e, "habit", off).await;
    }
    assert_eq!(
        streak(&pool, e).await,
        (0, 3),
        "a future-dated island is never the current streak"
    );
    assert_eq!(participation(&pool, e).await, (0, 3), "for both scanners");
}

// ---------------------------------------------------------------------------
// 3. What 0012 changed: participation moves the multiplier, the kind run does not
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn a_changed_kind_resets_the_exclusive_streak_but_not_the_multiplier(pool: PgPool) {
    // 0012's first measured row, reproduced exactly: 8 days present, alternating.
    // Before that migration the multiplier was read off the exclusive streak, so
    // this member earned 1.0x for showing up eight days running.
    let a = new_user(&pool, "alt").await;
    for off in -7..=0 {
        let kind = if off % 2 == 0 { "habit" } else { "denial" };
        log_on(&pool, a, kind, off).await;
    }
    let (kind_a, run_a, mult_a) = exclusive(&pool, a).await;
    assert_eq!(
        participation(&pool, a).await.0,
        8,
        "participation counts every kind"
    );
    assert_eq!(
        kind_a, "habit",
        "the exclusive streak follows the most recent kind"
    );
    assert_eq!(run_a, 1, "a run of ONE kind: alternating resets it");
    assert!(
        (mult_a - 1.25).abs() < 1e-9,
        "8 days of participation earns 1.25x, not 1.0x (got {mult_a})"
    );

    // Affirmations (drills) neither extend nor break the exclusive run, and they
    // do count as participation -- the member was present that day.
    let b = new_user(&pool, "drill").await;
    for off in -7..=0 {
        log_on(&pool, b, "habit", off).await;
    }
    log_on(&pool, b, "affirmation", -2).await;
    let (kind_b, run_b, mult_b) = exclusive(&pool, b).await;
    assert_eq!(
        (kind_b.as_str(), run_b),
        ("habit", 8),
        "a drill does not break the kind run"
    );
    assert!(
        (mult_b - 1.25).abs() < 1e-9,
        "8 participating days, one of them a drill: 1.25x (got {mult_b})"
    );

    // 0012's third row: 7 days present, then a 2-day gap. Not current, so no
    // multiplier -- and `longest` still reports the run that did happen.
    let c = new_user(&pool, "gapped").await;
    for off in -8..=-2 {
        log_on(&pool, c, "habit", off).await;
    }
    let (_, run_c, mult_c) = exclusive(&pool, c).await;
    assert_eq!(run_c, 0, "the run ended two days ago");
    assert!(
        (mult_c - 1.0).abs() < 1e-9,
        "a broken run earns no multiplier (got {mult_c})"
    );
    assert_eq!(
        participation(&pool, c).await,
        (0, 7),
        "the longest run survives the gap even though the current one is gone"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn the_multiplier_reads_participation_and_never_falls(pool: PgPool) {
    // 0012's guarantee, as a test: "no multiplier can fall on deploy and nobody
    // loses score for this change", because participation counts a superset of the
    // days any per-kind run counts and keeps the same today-or-yesterday rule.
    let patterns: [(&str, &[(i32, &str)]); 5] = [
        (
            "m_all_habit",
            &[
                (-7, "habit"),
                (-6, "habit"),
                (-5, "habit"),
                (-4, "habit"),
                (-3, "habit"),
                (-2, "habit"),
                (-1, "habit"),
                (0, "habit"),
            ],
        ),
        (
            "m_alternating",
            &[
                (-7, "habit"),
                (-6, "denial"),
                (-5, "habit"),
                (-4, "denial"),
                (-3, "habit"),
                (-2, "denial"),
                (-1, "habit"),
                (0, "denial"),
            ],
        ),
        (
            "m_denial_only",
            &[
                (-3, "denial"),
                (-2, "denial"),
                (-1, "denial"),
                (0, "denial"),
            ],
        ),
        (
            "m_drills_mixed",
            &[
                (-4, "affirmation"),
                (-3, "habit"),
                (-2, "denial"),
                (-1, "affirmation"),
                (0, "habit"),
            ],
        ),
        ("m_single_day", &[(0, "habit")]),
    ];

    for (name, logs) in patterns {
        let uid = new_user(&pool, name).await;
        for (off, kind) in logs {
            log_on(&pool, uid, kind, *off).await;
        }

        let part = participation(&pool, uid).await.0;
        let (kind, run, mult) = exclusive(&pool, uid).await;
        let from_participation: f64 = sqlx::query_scalar("select devotion_multiplier($1)::float8")
            .bind(part)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert!(
            (mult - from_participation).abs() < 1e-9,
            "{name}: the ONE multiplier must read participation ({part}), not the kind run ({run} of {kind})"
        );
        assert!(
            part >= run,
            "{name}: participation ({part}) is a superset of the kind run ({run}), so it can never be shorter"
        );
    }
}

/// `user_streak` (0001) and `user_participation_streak` (0012) are the same scan
/// written twice, and 0012's comment tells the next reader to "keep the two in
/// step". Nothing pinned that. This is the pin: on the same rows they must agree,
/// including the rows where a per-kind filter would change the answer.
#[sqlx::test(migrations = "./migrations")]
async fn the_two_day_scanners_agree(pool: PgPool) {
    let cases: [(&str, &[(i32, &str)]); 6] = [
        ("s_one_kind", &[(-2, "habit"), (-1, "habit"), (0, "habit")]),
        (
            "s_alternating",
            &[(-3, "habit"), (-2, "denial"), (-1, "habit"), (0, "denial")],
        ),
        ("s_denials_only", &[(-1, "denial"), (0, "denial")]),
        (
            "s_affirmations_only",
            &[(-1, "affirmation"), (0, "affirmation")],
        ),
        (
            "s_ended_yesterday",
            &[(-4, "habit"), (-3, "denial"), (-1, "affirmation")],
        ),
        (
            "s_gapped",
            &[(-6, "habit"), (-5, "habit"), (-1, "denial"), (0, "habit")],
        ),
    ];

    for (name, logs) in cases {
        let uid = new_user(&pool, name).await;
        for (off, kind) in logs {
            log_on(&pool, uid, kind, *off).await;
        }
        assert_eq!(
            streak(&pool, uid).await,
            participation(&pool, uid).await,
            "{name}: user_streak and user_participation_streak must agree on the same days"
        );
    }
}

// ---------------------------------------------------------------------------
// 4. The Rust half of the day rule
// ---------------------------------------------------------------------------
//
// `api.rs` promises that the Rust day agrees with the SQL day. Only the Rust half
// can be checked here (the SQL half is the table above), and it is checkable
// without a database.

#[test]
fn next_utc_midnight_is_the_boundary_the_sql_day_uses() {
    use streakforge_api::api::next_utc_midnight;
    use time::{Date, Month, OffsetDateTime, Time, UtcOffset};

    let at = |d: Date, h: u8, m: u8, s: u8, offset: UtcOffset| {
        OffsetDateTime::new_in_offset(d, Time::from_hms(h, m, s).unwrap(), offset)
    };
    let day = |y: i32, m: Month, d: u8| Date::from_calendar_date(y, m, d).unwrap();
    let utc = UtcOffset::UTC;
    let plus2 = UtcOffset::from_hms(2, 0, 0).unwrap();

    // A UTC instant: the next midnight is the following UTC date at 00:00.
    let cases = [
        (
            at(day(2026, Month::July, 15), 12, 0, 0, utc),
            day(2026, Month::July, 16),
        ),
        (
            at(day(2026, Month::July, 15), 23, 59, 59, utc),
            day(2026, Month::July, 16),
        ),
        (
            at(day(2026, Month::July, 15), 0, 0, 0, utc),
            day(2026, Month::July, 16),
        ),
        (
            at(day(2026, Month::December, 31), 23, 59, 59, utc),
            day(2027, Month::January, 1),
        ),
    ];
    for (now, want_date) in cases {
        let got = next_utc_midnight(now);
        assert_eq!(got.offset(), utc, "the boundary must be expressed in UTC");
        assert_eq!(
            (got.date(), got.time()),
            (want_date, Time::MIDNIGHT),
            "next UTC midnight after {now}"
        );
    }

    // A NON-UTC instant. The function is `pub` and its doc says "Next UTC
    // midnight", so the answer must not depend on the offset it is handed: a
    // +02 caller at 2026-07-15 01:30+02 is at 2026-07-14 23:30 UTC, whose next UTC
    // midnight is the 15th. Reading `now.date()` would say the 16th -- a day late,
    // and exactly the kind of disagreement with the SQL scanners this file exists
    // to catch.
    let local = at(day(2026, Month::July, 15), 1, 30, 0, plus2);
    let got = next_utc_midnight(local);
    assert_eq!(
        (got.date(), got.time(), got.offset()),
        (day(2026, Month::July, 15), Time::MIDNIGHT, utc),
        "next UTC midnight after {local} must follow the UTC day, not the caller's clock"
    );

    // And it must match the SQL boundary for the same instant: tomorrow's UTC date
    // at midnight is exactly one day after today's UTC date.
    let sql_next = day(2026, Month::July, 14) + time::Duration::days(1);
    assert_eq!(
        got.date(),
        sql_next,
        "the Rust day and the SQL day must agree"
    );
}
