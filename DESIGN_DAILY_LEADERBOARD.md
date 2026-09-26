# DESIGN — Daily Leaderboard Shows Nothing

**Date:** 2026-08-14
**Status:** Spec — awaiting review
**Author:** Hermes (coding profile)

## 1. Problem statement

On the live site (https://streakforge.polarisocial.xyz), the leaderboard **DAILY** tab
shows no entries even though users logged activity in the last 24 hours.

Reported by the maintainer:
> "the leaderboard isn't showing anything on the daily even though there are entries
> for the last 24 hours"

## 2. Investigation findings (verified 2026-08-14)

### 2.1 Live DB state (ThinkCentre, queried directly)

```
log_date  | count
-----------+-------
 2026-08-13 |     8
 2026-08-12 |     3
 2026-08-11 |     4

 utc_now = 2026-08-14 06:17:33

 count(*) where log_date = (now() at time zone 'utc')::date  = 0   (today logs)
 count(*) where logged_at >= now() - interval '24 hours'     = 6   (last 24h logs)
   → 4 habit, 2 affirmation
```

So **6 entries exist within the last 24 hours, 0 entries exist for the UTC calendar
day of "today"** (2026-08-14). At the time of the report (early morning UTC), the
most recent logs were from the *previous* UTC calendar day.

### 2.2 The query

`backend/migrations/0006_affirmations_on_leaderboard.sql` (and 0005/0003 before it):

```sql
create or replace function public.daily_leaderboard_weighted(p_limit int default 50)
returns table (...) language sql stable as $$
  select ...
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where l.log_date = (now() at time zone 'utc')::date          -- ← THE FILTER
    and l.kind in ('habit', 'denial', 'affirmation')
  group by p.id
  order by points desc, min(l.logged_at) asc
  limit p_limit;
$$;
```

`log_date` is a **generated column** (`0001_init.sql:22`):
```sql
log_date date generated always as ((logged_at at time zone 'utc')::date) stored
```

So `daily` = **UTC calendar day**, strictly. It is *not* a rolling 24-hour window.

### 2.3 API handler

`backend/src/api.rs:561-595` — `get_leaderboard` maps `"daily"` →
`select * from daily_leaderboard_weighted(50)`. No client-side filtering in
`web/src/routes/leaderboard/+page.svelte` (renders whatever the API returns).

### 2.4 UOTD ("Whiteboi of the Day") uses the same filter

`get_user_of_the_day` (`api.rs:597+`) also filters `l.log_date = today`. It is
empty for the same reason — the UOTD card on the leaderboard page (and landing
page) is blank early in the UTC day.

## 3. Root cause

**Semantics mismatch, not a crash.** "Daily" is implemented as "current UTC
calendar date". The user's mental model (and the phrase "entries for the last 24
hours") is a **rolling 24-hour window**. Between UTC midnight and the first log of
the new UTC day, the daily board is empty even when there is recent activity.

This is worse for the StreakForge audience because:
- The site's most active period is **evening US time = 00:00–06:00 UTC** — exactly
  when the daily board is guaranteed empty (no one has logged "today" UTC yet).
- Early-morning UTC visitors (US evening) see an empty DAILY tab, which reads as
  "the site is dead" even when activity is heavy.

## 4. Options

### Option A — Rolling 24-hour window (recommended)

Change the daily filter from UTC-calendar-day to a rolling window:

```sql
where l.logged_at >= now() - interval '24 hours'
  and l.kind in ('habit', 'denial', 'affirmation')
```

- Matches the user's stated expectation ("entries for the last 24 hours").
- Always shows the most recent activity; no dead-zone after UTC midnight.
- UOTD "Whiteboi of the Day" becomes "last 24 hours" too (rename copy or keep).
- Straightforward: new migration 0007 redefining `daily_leaderboard_weighted` +
  the inline UOTD query in `api.rs`.

**Trade-off:** "Daily" becomes a floating window — the same user could appear on
two consecutive "days" for overlapping 24h spans. For a kink-denial leaderboard
this is fine (it's a live/activity board, not an accounting ledger).

### Option B — Keep UTC calendar day, add an empty-state message

Keep the current filter. Add a "No logs yet today — be the first to waste/deny"
empty state on the DAILY tab (and UOTD hidden when null).

- Honest to "daily = calendar day", but **does not fix the user's complaint** —
  the board is still empty early-morning UTC.
- Minimal change (frontend only).

### Option C — Hybrid: calendar-day with 24h fallback

Show today's UTC logs; **if none exist, fall back to the last 24h**. Label stays
DAILY.

- Never empty, but confusing: the content definition silently changes.
- More complex SQL/UI.

### Option D — Client-local "today" semantics

Interpret "daily" in the viewer's local timezone (client sends its offset).
- Nice in theory, but adds API surface (tz param), breaks caching, and is
  overkill for a niche leaderboard. Rejected.

## 5. Recommendation

**Option A (rolling 24h)** — **IMPLEMENTED 2026-08-14** with these details:

1. **Migration 0007** (`backend/migrations/0007_rolling_windows.sql`):
   - `daily_leaderboard_weighted` now filters `logged_at >= now() - interval '24 hours'`.
   - `weekly_leaderboard_weighted` now filters `logged_at >= now() - interval '7 days'`
     (same rolling behavior, per maintainer direction).
   - Same return signature, so the API handler was untouched.
2. **UOTD query** in `backend/src/api.rs` — changed to the same 24h window;
   frontend copy now reads "points · 24h" (leaderboard + landing).
3. **Empty state** — DAILY/WEEKLY/ALL-TIME tab shows
   "NO LOGS IN THIS WINDOW — BE THE FIRST TO WASTE A LOAD" when entries is empty.
4. **Tests** — `daily_uses_rolling_24h_window`: a 12h-old log appears on daily
   and weekly; a 3d-old log appears on weekly but NOT daily. Backend 18/18 green.

The tab label stays **DAILY** (per maintainer decision); the window is rolling.

## 6. Open questions for the maintainer

1. **Label semantics:** keep the tab label "DAILY" for a rolling 24h window, or
   rename to "24H"? (I lean keep "DAILY" — the site's vibe favors short labels,
   and "daily" colloquially = "recent".)
2. **UOTD copy:** keep "points today" or change to "points · 24h"? UOTD is also
   shown on the landing page hero.
3. Should the **weekly** board also become rolling-7-days for consistency? (I lean
   **no** — weekly resets Monday, which is fine and matches calendar expectations;
   the dead-zone only bites at daily granularity.)

## 7. Files touched (if approved)

- `backend/migrations/0007_daily_24h_window.sql` (new)
- `backend/src/api.rs` — UOTD inline query window
- `web/src/routes/leaderboard/+page.svelte` — empty state + UOTD copy
- `web/src/routes/+page.svelte` — landing UOTD copy (if changed)
- `backend/tests/integration.rs` — daily-window test
- `web/src/lib/...` tests if empty-state is extracted
- `STATUS.md` / `README.md` — note the daily = 24h semantics
