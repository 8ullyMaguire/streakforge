//! GS-3 — cookie consent, enforced by the API and not by the toast.
//!
//! Estate standard: `10-Projects/2026-10-16-guest-access-trust-and-self-governance.md` §3.
//!
//! The gate lives here rather than in the Svelte store because a client-side
//! check is a suggestion. The toast is the courtesy; this middleware is the
//! refusal. Until consent is accepted the site is anonymous **read-only**: every
//! mutating method is refused with `403 {"consent":"required"}`, and every read
//! route stays open — the public board is public, which is the whole point of it.
//!
//! Where the "yes" lives, in the three places the standard asks for:
//!
//! * the **session** — a signed cookie over a server-side row. The gate reads
//!   this, and it works for a visitor who has no account to stamp yet;
//! * `profiles.consented_at` / `.consent_version` — the half that survives the
//!   browser, so a returning user on a new device is not asked again;
//! * `audit_log` — the half that survives the person, because "when, and to
//!   which version" has to be answerable after the fact.
//!
//! A "no" is honoured: no row, no persisted flag, no session entry, and the API
//! keeps refusing mutations. A gate whose no is not honoured is a dark pattern
//! with a compliance fig leaf — and the client half (`web/src/lib/consent.ts`)
//! keeps the refusal in `sessionStorage`, so it re-asks next visit rather than
//! nagging in a loop.

use axum::extract::{Request, State};
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Json, Response};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_sessions::Session;
use uuid::Uuid;

use crate::auth::{SessionUser, SESSION_USER_KEY};
use crate::error::ApiResult;
use crate::AppState;

/// Session key holding the accepted version (an `i32`).
pub const SESSION_KEY: &str = "consent_version";
/// Session key holding the optional categories the visitor accepted.
pub const CATEGORIES_KEY: &str = "consent_categories";
/// Session key holding the fact of a refusal, so the API knows the difference
/// between "never asked" and "asked and said no".
pub const REFUSED_KEY: &str = "consent_refused";

pub const ACCEPTED_ACTION: &str = "consent.accepted";
pub const REFUSED_ACTION: &str = "consent.refused";

/// The optional categories, in the order the toast shows them. `essential` is
/// always on and is not a choice: the site does not work without a session.
pub const OPTIONAL_CATEGORIES: [&str; 2] = ["analytics", "personalisation"];

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub accept: bool,
    /// Empty means "only what is essential" — the default the standard sets.
    #[serde(default)]
    pub categories: Vec<String>,
}

/// Paths allowed to mutate without consent.
///
/// * `POST /api/consent` is how consent is given; gating it would be a locked
///   door with the key inside it.
/// * `POST /api/auth/logout` ends a session, and ending a session must never
///   need permission.
///
/// The comparison strips `/api` first, because this layer sits on the router that
/// `nest` mounted under `/api` and axum hands the nested router the URI **without
/// the prefix**. Writing the full paths here and normalizing once is deliberate:
/// the list is meant to be read as URLs by someone looking at the route table, and
/// the first version of this function compared full paths against stripped ones,
/// which exempted nothing and turned the consent endpoint into a locked door with
/// the key inside it — caught by `an_unknown_category_is_not_recorded`, which got
/// a 403 where it expected its own refusal echoed back.
///
/// **Login is exempt from the middleware but not from the rule.** It has to be:
/// a returning user on a device that was never asked must be able to sign in on
/// the strength of the account's own record, and this layer cannot see the
/// account — it would refuse before the handler that knows. So the check lives in
/// `auth::login` (`consented_at is not null` on the profile, else the same 403),
/// which is the one place where "no session consent" has a second answer.
///
/// Exempting it here does **not** open a hole: `login` returns the identical
/// refusal shape, so the API contract is the same either way, and the test
/// `an_account_that_never_consented_cannot_sign_in_without_asking` pins it.
pub fn exempt(path: &str) -> bool {
    matches!(
        path.strip_prefix("/api").unwrap_or(path),
        "/consent" | "/auth/logout" | "/auth/login"
    )
}

/// Methods that need consent. Reads (`GET`, `HEAD`, `OPTIONS`) never do.
pub fn is_mutation(method: &Method) -> bool {
    matches!(
        *method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    )
}

/// Has this session accepted the version the site currently asks for?
///
/// `>=` rather than `==`: a visitor who accepted version 2 has also accepted
/// version 1, and a downgrade of the configured version must not re-ask.
pub async fn accepted(session: &Session, version: i32) -> bool {
    matches!(session.get::<i32>(SESSION_KEY).await, Ok(Some(v)) if v >= version)
}

/// The 403 every mutating route answers with until consent is given.
pub fn refusal(version: i32) -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({"consent": "required", "version": version})),
    )
        .into_response()
}

/// Mark this session as consenting, and record it.
pub async fn grant(
    state: &AppState,
    session: &Session,
    categories: &[String],
    profile_id: Option<Uuid>,
) -> ApiResult<()> {
    let version = state.cfg.consent_version;
    session
        .insert(SESSION_KEY, version)
        .await
        .map_err(crate::error::ApiError::from)?;
    session
        .insert(CATEGORIES_KEY, categories.to_vec())
        .await
        .map_err(crate::error::ApiError::from)?;
    session.remove::<bool>(REFUSED_KEY).await.ok();

    // The identity half. `coalesce` keeps the FIRST consent: a second accept on
    // another device must not rewrite when the person actually consented.
    if let Some(id) = profile_id {
        sqlx::query(
            "update profiles
                set consented_at = coalesce(consented_at, now()), consent_version = $2
              where id = $1",
        )
        .bind(id)
        .bind(version)
        .execute(&state.pool)
        .await?;
    }

    // A failure to audit must not refuse a consent the visitor has given; the
    // session already carries it and the next `grant` retries the stamp.
    if let Err(err) = crate::audit::record(
        &state.pool,
        profile_id,
        ACCEPTED_ACTION,
        json!({
            "version": version,
            "categories": categories,
            "session": true,
        }),
    )
    .await
    {
        tracing::warn!(error = %err, "consent accepted but the audit row was not written");
    }
    Ok(())
}

/// The middleware. Reads the session, refuses mutations without consent.
pub async fn require_consent(
    session: Session,
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    if !state.cfg.consent_required
        || !is_mutation(req.method())
        || exempt(req.uri().path())
        || accepted(&session, state.cfg.consent_version).await
    {
        return next.run(req).await;
    }
    refusal(state.cfg.consent_version)
}

/// GET /api/consent — what the toast needs to decide whether to show itself.
///
/// Public and cheap: it must answer for a visitor with no session at all, which
/// is the visitor the toast exists for.
pub async fn get(session: Session, State(state): State<AppState>) -> Json<Value> {
    let version = state.cfg.consent_version;
    let is_accepted = accepted(&session, version).await;
    let refused = matches!(session.get::<bool>(REFUSED_KEY).await, Ok(Some(true)));
    let categories = session
        .get::<Vec<String>>(CATEGORIES_KEY)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
    Json(json!({
        "version": version,
        "required": state.cfg.consent_required,
        "accepted": is_accepted,
        "refused": refused,
        "categories": category_state(&categories, is_accepted),
    }))
}

/// POST /api/consent — record the choice. `{"accept": true|false}`.
///
/// Both answers are 200: a refusal is not a client error, it is a decision the
/// site honours. The response says which one happened so the toast can close
/// without guessing from a status code.
pub async fn post(
    State(state): State<AppState>,
    session: Session,
    Json(choice): Json<Choice>,
) -> ApiResult<Json<Value>> {
    let version = state.cfg.consent_version;
    let profile_id = session
        .get::<SessionUser>(SESSION_USER_KEY)
        .await
        .ok()
        .flatten()
        .map(|u| u.id);

    if choice.accept {
        // Unknown names are dropped rather than stored: the record is what was
        // accepted, and a category this site does not have was not.
        let accepted: Vec<String> = OPTIONAL_CATEGORIES
            .iter()
            .filter(|name| choice.categories.iter().any(|c| c == *name))
            .map(|name| (*name).to_string())
            .collect();
        grant(&state, &session, &accepted, profile_id).await?;
        return Ok(Json(json!({
            "consent": "accepted",
            "version": version,
            "categories": category_state(&accepted, true),
        })));
    }

    // Refusal: nothing persisted. The session marker is memory inside a session
    // row that expires; the client keeps its own per-tab flag.
    session.remove::<i32>(SESSION_KEY).await.ok();
    session.remove::<Vec<String>>(CATEGORIES_KEY).await.ok();
    session.insert(REFUSED_KEY, true).await.ok();
    if let Err(err) = crate::audit::record(
        &state.pool,
        profile_id,
        REFUSED_ACTION,
        json!({ "version": version }),
    )
    .await
    {
        tracing::warn!(error = %err, "consent refused but the audit row was not written");
    }
    Ok(Json(json!({
        "consent": "refused",
        "version": version,
    })))
}

fn category_state(accepted: &[String], any_accepted: bool) -> Value {
    let mut map = serde_json::Map::new();
    map.insert("essential".into(), json!(true));
    for name in OPTIONAL_CATEGORIES {
        map.insert(
            name.into(),
            json!(any_accepted && accepted.iter().any(|c| c == name)),
        );
    }
    Value::Object(map)
}
