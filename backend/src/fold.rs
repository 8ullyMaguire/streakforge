//! GS-2 — folding a guest's rows into the account they register.
//!
//! Two rules make this safe, and both come from the standard (§2):
//!
//! 1. **Schema-driven completeness.** The fold asks `pg_constraint` for every
//!    column that references `profiles(id)` and **fails if it finds one it does not
//!    handle**. A hand-kept list of tables drifts the first time a migration lands;
//!    a fold that silently misses a column surfaces months later as "my logs
//!    disappeared". The check *is* the feature.
//! 2. **One transaction, idempotent.** Everything or nothing, and a second fold for
//!    the same guest is a no-op rather than a double credit (the guest row carries
//!    `merged_into`, so "already folded" is a fact in the data, not a hope).
//!
//! Firewall: the fold grants **tenure** and never **score**. Here tenure is the
//! whole feature — the streak is computed from `habit_logs`, so moved logs mean the
//! new account's first day is not day zero. There is no trust ladder on this site
//! yet, so `merge.grants_trust` has nothing to grant and stays false by having
//! nothing to do; when GS-4 lands, this is the function that must not touch it.

use std::fmt;

use sqlx::{PgConnection, Row};
use uuid::Uuid;

/// Columns that hold the guest's own content and are re-parented.
///
/// Every entry here must also appear in the `pg_constraint` walk below, or the walk
/// will report it as unhandled — that duplication is deliberate: the list is what
/// the code does, and the walk is what proves the list is still complete.
const FOLDED: &[(&str, &str)] = &[
    ("habit_logs", "user_id"),
    ("lock_sessions", "user_id"),
    // A cage check is a claim about your own cage, and the hours it earns are yours:
    // dropping them at registration would answer "where did my verified time go"
    // with silence. A fold collision is impossible here — the account was created in
    // this same transaction and cannot already hold a check for the guest's date.
    ("cage_checks", "user_id"),
    // Attestations are votes a guest cast on OTHER people's checks, and 0016 counts
    // them towards `contributions`, i.e. towards tenure. The firewall folds tenure
    // and never score, and dropping these would silently reset a member's progress
    // to level 1 the moment they registered. So they move; the `unique (check_id,
    // attester_id)` index cannot collide with an account that did not exist a
    // statement ago.
    ("cage_attestations", "attester_id"),
    // An affirmation is a line the guest wrote for the drill, and the drill is one deck
    // for everyone: leaving it on the tombstone would delete a stranger's favourite
    // mantra at the moment its author registered. Nothing is scored here — the deck has
    // no author ranking — so folding it moves no credit, which is what the firewall
    // requires. It also cannot collide: `affirmations_one_live_per_line` covers `live`
    // rows only, and a guest is level 0, so everything a guest wrote is `pending`.
    ("affirmations", "author_id"),
];

/// Columns that reference `profiles(id)` and are deliberately **not** folded.
///
/// * `audit_log.profile_id` — the audit trail. Rewriting it would move the record of
///   what the guest *did* onto the new account, which is precisely the laundering
///   the standard's firewall forbids one layer up; the row keeps pointing at the
///   tombstone, which now says where the history went.
/// * `profiles.merged_into` — the tombstone's own pointer, written by this fold.
/// * `flags.reporter_id` — a flag is an opinion about someone else's content, not
///   the guest's own. Moving it would let a person register and re-cast the verdict
///   under a new name (the unique index is per reporter). It keeps pointing at the
///   tombstone, whose trust level is 0, so it still counts as one untrusted voice.
/// * `moderation_actions.actor_id` — an action log. It records who acted; rewriting
///   it would move the record of a moderation decision onto an account that did not
///   make it, which is the audit-laundering case `audit_log` exists to prevent.
const EXEMPT: &[(&str, &str)] = &[
    ("audit_log", "profile_id"),
    ("profiles", "merged_into"),
    ("flags", "reporter_id"),
    ("moderation_actions", "actor_id"),
    // `affirmations.reviewed_by` — who admitted a line to the deck. Same reasoning as
    // `moderation_actions.actor_id`: it is the record of a decision about someone else's
    // content, and a guest cannot hold the level that makes it (level 2 needs active
    // days), so the only rows that ever point here are from an account, not a tombstone.
    ("affirmations", "reviewed_by"),
];

#[derive(Debug)]
pub enum FoldError {
    /// A table references `profiles(id)` and this fold does not handle it. The
    /// caller must refuse the registration rather than half-move a person's data.
    Unhandled { table: String, column: String },
    Db(sqlx::Error),
}

impl fmt::Display for FoldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FoldError::Unhandled { table, column } => write!(
                f,
                "the guest fold does not handle {table}.{column}, which references \
                 profiles(id); refusing to fold rather than leave the rows behind"
            ),
            FoldError::Db(e) => write!(f, "guest fold failed: {e}"),
        }
    }
}

impl std::error::Error for FoldError {}

impl From<sqlx::Error> for FoldError {
    fn from(e: sqlx::Error) -> Self {
        FoldError::Db(e)
    }
}

#[derive(Debug, Default, Clone)]
pub struct FoldReport {
    /// False when the guest had already been folded (a retry, or a double submit).
    pub folded: bool,
    /// Rows re-parented, per table, in the order they were moved.
    pub moved: Vec<(String, u64)>,
}

/// Move every row the guest authored onto `account_id`, then tombstone the guest.
///
/// Runs on a caller-provided connection so the whole registration — the new profile,
/// the fold, and the consent stamp — is one transaction: a fold that fails must take
/// the account with it, otherwise the person is left with an empty account and their
/// streak still attached to an identity they cannot log into.
pub async fn fold_guest(
    conn: &mut PgConnection,
    guest_id: Uuid,
    account_id: Uuid,
) -> Result<FoldReport, FoldError> {
    // Idempotent: `merged_into` is written in the same transaction as the moves, so
    // a second call sees it and does nothing.
    let already: Option<Option<Uuid>> =
        sqlx::query_scalar("select merged_into from profiles where id = $1")
            .bind(guest_id)
            .fetch_optional(&mut *conn)
            .await?;
    match already {
        None => {
            return Err(FoldError::Db(sqlx::Error::RowNotFound));
        }
        Some(Some(_)) => return Ok(FoldReport::default()),
        Some(None) => {}
    }

    for (table, column) in referencing_columns(&mut *conn).await? {
        if EXEMPT.contains(&(table.as_str(), column.as_str())) {
            continue;
        }
        if !FOLDED.contains(&(table.as_str(), column.as_str())) {
            return Err(FoldError::Unhandled { table, column });
        }
    }

    let mut moved = Vec::with_capacity(FOLDED.len());
    for (table, column) in FOLDED {
        // Table and column come from the constant above, never from input.
        let sql = format!("update public.{table} set {column} = $2 where {column} = $1");
        let result = sqlx::query(&sql)
            .bind(guest_id)
            .bind(account_id)
            .execute(&mut *conn)
            .await?;
        moved.push((format!("{table}.{column}"), result.rows_affected()));
    }

    // Tombstone, not delete: the audit rows point here, and a deletion request is
    // answered from them. The username is released so the person may register it.
    let tombstone = format!("folded_{}", &guest_id.simple().to_string()[..12]);
    sqlx::query(
        "update profiles
            set is_guest = true,
                merged_into = $2,
                username = $3,
                display_name = null,
                avatar_url = null,
                social_url = null
          where id = $1",
    )
    .bind(guest_id)
    .bind(account_id)
    .bind(&tombstone)
    .execute(&mut *conn)
    .await?;

    Ok(FoldReport {
        folded: true,
        moved,
    })
}

/// Every `(table, column)` in `public` that references `public.profiles(id)`.
async fn referencing_columns(conn: &mut PgConnection) -> Result<Vec<(String, String)>, FoldError> {
    let rows = sqlx::query(
        r#"
        select c.relname::text as table_name, a.attname::text as column_name
        from pg_constraint con
        join pg_class c on c.oid = con.conrelid
        join pg_namespace n on n.oid = c.relnamespace
        join pg_class rc on rc.oid = con.confrelid
        join pg_namespace rn on rn.oid = rc.relnamespace
        join unnest(con.conkey) as k(attnum) on true
        join pg_attribute a on a.attrelid = c.oid and a.attnum = k.attnum
        where con.contype = 'f'
          and n.nspname = 'public'
          and rn.nspname = 'public'
          and rc.relname = 'profiles'
        order by 1, 2
        "#,
    )
    .fetch_all(&mut *conn)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            (
                r.get::<String, _>("table_name"),
                r.get::<String, _>("column_name"),
            )
        })
        .collect())
}

/// The tables this fold handles, for tests and for the operator-facing list.
pub fn folded_columns() -> &'static [(&'static str, &'static str)] {
    FOLDED
}

/// The columns it deliberately leaves alone, with no reason attached (the reasons
/// are in the doc comment above — one place to keep them true).
pub fn exempt_columns() -> &'static [(&'static str, &'static str)] {
    EXEMPT
}
