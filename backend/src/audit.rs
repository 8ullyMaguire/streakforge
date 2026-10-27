//! The audit log: who did what, when — answerable after the fact.
//!
//! Written for GS-3 (when someone consented, and to which version). GS-1's MAU
//! count and GS-5's user-visible decision records read the same table, which is
//! why `action` is a free string and `detail` carries the shape of the event
//! rather than one event's fields becoming columns.
//!
//! A write that fails is an error the caller may choose to ignore: refusing a
//! sign-in because the audit row could not be written is worse than a missing
//! row, and the caller decides which of those it can live with.

use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

/// Append one event. `profile_id` is `None` for an event with no account behind
/// it (an anonymous consent, a refused one).
pub async fn record(
    pool: &PgPool,
    profile_id: Option<Uuid>,
    action: &str,
    detail: Value,
) -> Result<(), sqlx::Error> {
    sqlx::query("insert into audit_log (profile_id, action, detail) values ($1, $2, $3)")
        .bind(profile_id)
        .bind(action)
        .bind(detail)
        .execute(pool)
        .await
        .map(|_| ())
}

/// Distinct accounts with at least one recorded action inside the window.
///
/// GS-1's definition, kept verbatim: MAU counts *accounts that acted*, not
/// visitors that arrived, because the cap it feeds exists to limit how much
/// unowned content the site is carrying at once.
pub async fn monthly_active_accounts(pool: &PgPool, window_days: i32) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "select count(distinct profile_id) from audit_log
         where profile_id is not null and at >= now() - make_interval(days => $1)",
    )
    .bind(window_days.max(1))
    .fetch_one(pool)
    .await
}
