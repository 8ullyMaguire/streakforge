# KPI Dashboard Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Add a public community KPI dashboard at `/kpi` — one page that shows the site's health: loads wasted/denied, affirmations, denial rate, active users, new users, currently-locked count, total lock hours, a 30-day activity trend, and the top weekly whitebois.

**Architecture:** One new migration `0008_kpis.sql` defines two SQL functions (`kpi_totals()`, `kpi_trend(n)`). A new public endpoint `GET /api/kpi` in the existing axum API composes totals + trend + top-weekly (reusing `weekly_leaderboard_weighted`). Frontend adds a SvelteKit route `/kpi` that renders cards and a pure-CSS stacked bar chart (no chart library). Public, no auth — consistent with the landing page showing public counters; the "community is alive" number is the feature.

**Tech Stack:** Rust/axum + sqlx (Postgres), SvelteKit (Svelte 5, `$state`), vitest for frontend tests, `cargo test --test integration` for backend.

---

## Design decisions (open for review)

| Decision | Choice | Why |
|---|---|---|
| Route | `/kpi` (public) | Landing already shows public totals; a live KPI page is a growth/community signal. No auth gate. |
| Denial rate | `denied / (wasted + denied) * 100`, rounded 1dp | "Of all loads committed, what % were denied" — the thematic north-star. |
| Trend window | Last 30 UTC days, per-day counts by kind | Simple, readable bars; `log_date` generated column exists (0001) — no new storage. |
| Trend axis | UTC day (`log_date`) | Trend is a coarse daily view; the rolling-window fix (0007) only matters for leaderboard/UOTD live windows. |
| Lock hours | `sum(extract(epoch from (coalesce(unlocked_at, now()) - locked_at)))/3600` all-time | Open sessions count as "until now". |
| Chart | Pure CSS stacked bars (flex + `height:%`) | No new dep; theme is brutalist/dark, fits. |
| Copy | "loads wasted" / "loads denied" — **never `load$`** (2026-08-13 correction) | Hard rule from maintainer. |
| Nav | Footer link (like "Install the app"), NOT BottomNav (5 items fixed) | Keep mobile nav stable. |

**Open questions for maintainer:**
1. Public or gated? Default **public** (matches landing counters). A gated option = add `require_user` + an admin flag — bigger change, not in scope unless requested.
2. Should the trend be per-kind stacked bars, or total-only? Default **stacked** (wasted/denied/affirmations visible).
3. "New whitebois" window: last 7 days vs last 30? Default **7 days** (matches "new this week" mental model).

---

## Current state (verified 2026-08-30)

- Backend: `backend/src/api.rs` (835 lines, handlers + DTOs), `backend/src/main.rs` router (127 lines), migrations 0001–0007.
- `habit_logs` has `kind` check-constraint: `habit | affirmation | denial` (0005), `log_date` generated UTC date (0001).
- `lock_sessions (user_id, locked_at, unlocked_at, reason)` (0005) — `unlocked_at is null` = currently locked.
- `weekly_leaderboard_weighted(limit)` already returns `(user_id, username, display_name, avatar_url, points, waste_count, denial_count, affirmation_count, last_log_at)` — reuse for top-weekly.
- Backend tests: `backend/tests/integration.rs` (`#[sqlx::test(migrations = "./migrations")]`), 18 green. Frontend vitest in `web/src/lib/*.test.ts`, 47 green.
- Existing leaderboard handler shape to copy: `get_leaderboard` (`api.rs:561`) — tuple row decode + enumerate → entries.

---

## Task 1: Migration 0008 — KPI SQL functions + backend test

**Objective:** Two SQL functions that answer all KPI questions in one call each.

**Files:**
- Create: `backend/migrations/0008_kpis.sql`
- Test: `backend/tests/integration.rs`

**Step 1: Write failing test** — append to `backend/tests/integration.rs`:

```rust
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
        sqlx::query("insert into habit_logs (user_id, kind) values ($1, 'habit')").bind(a).execute(&pool).await.unwrap();
    }
    sqlx::query("insert into habit_logs (user_id, kind) values ($1, 'denial')").bind(a).execute(&pool).await.unwrap();
    sqlx::query("insert into habit_logs (user_id, kind, logged_at) values ($1, 'affirmation', now() - interval '2 days')")
        .bind(a).execute(&pool).await.unwrap();
    // b: 1 waste 30 hours ago (not in 24h window, inside trend)
    sqlx::query("insert into habit_logs (user_id, kind, logged_at) values ($1, 'habit', now() - interval '30 hours')")
        .bind(b).execute(&pool).await.unwrap();

    let (tw, td, ta, tu, active, new7, locked, lock_h, rate): (i64, i64, i64, i64, i64, i64, i64, f64, f64) =
        sqlx::query_as("select * from kpi_totals()").fetch_one(&pool).await.unwrap();
    assert_eq!(tw, 3);           // 2 + 1 (30h log counts all-time)
    assert_eq!(td, 1);
    assert_eq!(ta, 1);
    assert_eq!(tu, 2);
    assert_eq!(active, 1);       // only a logged within 24h
    assert_eq!(new7, 1);         // only a created within 7d
    assert_eq!(locked, 0);
    assert!(lock_h == 0.0);
    assert!((rate - 25.0).abs() < 0.1); // 1 denied / (3+1) = 25%

    // an active lock adds to currently_locked + lock hours > 0
    sqlx::query("insert into lock_sessions (user_id, locked_at) values ($1, now() - interval '2 hours')")
        .bind(a).execute(&pool).await.unwrap();
    let (locked, lock_h): (i64, f64) =
        sqlx::query_as("select currently_locked, total_lock_hours from kpi_totals()").fetch_one(&pool).await.unwrap();
    assert_eq!(locked, 1);
    assert!(lock_h >= 2.0, "lock_h={lock_h}");

    // trend: 30 rows, today has 3 (2 waste + 1 denial), 2 days ago has 1 affirmation
    let trend: Vec<(String, i64, i64, i64)> =
        sqlx::query_as("select * from kpi_trend(30)").fetch_all(&pool).await.unwrap();
    assert_eq!(trend.len(), 30);
    let today = trend.last().unwrap().clone();
    assert_eq!(today.1, 2); // wasted today
    assert_eq!(today.2, 1); // denied today
    let two_days_ago = trend[trend.len() - 3].clone();
    assert_eq!(two_days_ago.3, 1); // affirmation 2 days ago
}
```

**Step 2: Run to verify failure**

```bash
cd backend && DATABASE_URL=postgres://streakforge:<pw>@127.0.0.1:5432/streakforge_test cargo test --test integration kpi_totals_and_trend
```

Expected: FAIL — `function kpi_totals() does not exist` / `function kpi_trend(integer) does not exist`.

**Step 3: Create `backend/migrations/0008_kpis.sql`**

```sql
-- 0008_kpis.sql — community KPI dashboard functions
--
-- kpi_totals(): one row of headline numbers for the /kpi page.
-- kpi_trend(n): per-UTC-day counts of each kind for the last n days (n default 30).
--
-- Note: denial_rate = denied / (wasted + denied) * 100  ("of all loads committed,
-- what % were denied"). Zero-divided -> 0.

create or replace function public.kpi_totals()
returns table (
  total_wasted bigint,
  total_denied bigint,
  total_affirmations bigint,
  total_users bigint,
  active_24h bigint,
  new_7d bigint,
  currently_locked bigint,
  total_lock_hours numeric,
  denial_rate numeric
) language sql stable as $$
  select
    (select count(*) from public.habit_logs where kind = 'habit'),
    (select count(*) from public.habit_logs where kind = 'denial'),
    (select count(*) from public.habit_logs where kind = 'affirmation'),
    (select count(*) from public.profiles),
    (select count(distinct user_id) from public.habit_logs where logged_at >= now() - interval '24 hours'),
    (select count(*) from public.profiles where created_at >= now() - interval '7 days'),
    (select count(*) from public.lock_sessions where unlocked_at is null),
    (select coalesce(sum(extract(epoch from (coalesce(unlocked_at, now()) - locked_at)) / 3600.0), 0)
     from public.lock_sessions),
    (select case
       when (select count(*) from public.habit_logs where kind in ('habit','denial')) = 0 then 0
       else round(
         (select count(*) from public.habit_logs where kind = 'denial')::numeric
         / (select count(*) from public.habit_logs where kind in ('habit','denial')) * 100, 1)
     end)
$$;

create or replace function public.kpi_trend(p_days int default 30)
returns table (day text, wasted bigint, denied bigint, affirmations bigint)
language sql stable as $$
  with days as (
    select generate_series(
      (now() at time zone 'utc')::date - (p_days - 1),
      (now() at time zone 'utc')::date,
      interval '1 day'
    )::date as day
  )
  select to_char(d.day, 'YYYY-MM-DD') as day,
         count(l.id) filter (where l.kind = 'habit')::bigint as wasted,
         count(l.id) filter (where l.kind = 'denial')::bigint as denied,
         count(l.id) filter (where l.kind = 'affirmation')::bigint as affirmations
  from days d
  left join public.habit_logs l on l.log_date = d.day
  group by d.day
  order by d.day;
$$;
```

**Step 4: Run test to verify pass**

```bash
DATABASE_URL=postgres://streakforge:<pw>@127.0.0.1:5432/streakforge_test cargo test --test integration kpi_totals_and_trend
```

Expected: PASS (and full suite stays green — 19 backend tests).

**Step 5: Commit**

```bash
git add backend/migrations/0008_kpis.sql backend/tests/integration.rs
git commit -m "feat: add KPI SQL functions (kpi_totals, kpi_trend) + test"
```

---

## Task 2: Backend API — DTOs, handler, route

**Objective:** `GET /api/kpi` returns `{ totals, trend, top_weekly }`.

**Files:**
- Modify: `backend/src/api.rs` (append DTOs + handler near leaderboard code, ~line 595)
- Modify: `backend/src/main.rs` (route registration)

**Step 1: Write failing test** — none practical at HTTP layer (existing integration tests exercise SQL directly; the handler is a thin mapper like `get_leaderboard`). Verification = compile + curl in Task 6. Documented deviation from pure TDD: handler shape is identical to the already-tested `get_leaderboard`.

**Step 2: Add DTOs + handler to `backend/src/api.rs`** (after `get_leaderboard`, before `get_user_of_the_day`):

```rust
#[derive(Debug, Serialize)]
pub struct KpiTotalsResponse {
    pub total_wasted: i64,
    pub total_denied: i64,
    pub total_affirmations: i64,
    pub total_users: i64,
    pub active_24h: i64,
    pub new_7d: i64,
    pub currently_locked: i64,
    pub total_lock_hours: f64,
    pub denial_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct KpiTrendPoint {
    pub day: String,
    pub wasted: i64,
    pub denied: i64,
    pub affirmations: i64,
}

#[derive(Debug, Serialize)]
pub struct KpiResponse {
    pub totals: KpiTotalsResponse,
    pub trend: Vec<KpiTrendPoint>,
    pub top_weekly: Vec<LeaderboardEntry>,
}

/// GET /api/kpi — public community KPIs (no auth).
pub async fn get_kpi(State(state): State<AppState>) -> ApiResult<Json<KpiResponse>> {
    let (tw, td, ta, tu, active, new7, locked, lock_h, rate): (i64, i64, i64, i64, i64, i64, i64, f64, f64) =
        sqlx::query_as("select * from kpi_totals()")
            .fetch_one(&state.pool)
            .await?;

    let trend_rows: Vec<(String, i64, i64, i64)> =
        sqlx::query_as("select * from kpi_trend(30)")
            .fetch_all(&state.pool)
            .await?;
    let trend = trend_rows
        .into_iter()
        .map(|(day, wasted, denied, affirmations)| KpiTrendPoint {
            day,
            wasted,
            denied,
            affirmations,
        })
        .collect();

    let rows: Vec<(uuid::Uuid, String, Option<String>, Option<String>, i64, i64, i64, i64, Option<OffsetDateTime>)> =
        sqlx::query_as("select * from weekly_leaderboard_weighted(10)")
            .fetch_all(&state.pool)
            .await?;
    let top_weekly = rows
        .into_iter()
        .enumerate()
        .map(|(i, (uid, uname, dname, av, points, waste_count, denial_count, affirmation_count, last))| {
            LeaderboardEntry {
                rank: (i + 1) as i64,
                user_id: uid,
                username: uname,
                display_name: dname,
                avatar_url: av,
                points,
                waste_count,
                denial_count,
                affirmation_count,
                last_log_at: last
                    .map(|t| t.format(&time::format_description::well_known::Rfc3339).unwrap_or_default()),
            }
        })
        .collect();

    Ok(Json(KpiResponse {
        totals: KpiTotalsResponse {
            total_wasted: tw,
            total_denied: td,
            total_affirmations: ta,
            total_users: tu,
            active_24h: active,
            new_7d: new7,
            currently_locked: locked,
            total_lock_hours: lock_h,
            denial_rate: rate,
        },
        trend,
        top_weekly,
    }))
}
```

**Step 3: Register route in `backend/src/main.rs`** — inside `api_router` (after `.route("/total", ...)`):

```rust
.route("/kpi", get(streakforge_api::api::get_kpi))
```

**Step 4: Verify compile + format**

```bash
cd backend && cargo build 2>&1 | tail -5 && cargo fmt --check
```

Expected: no errors. (No new deps needed — `time`, `sqlx`, `uuid` already used.)

**Step 5: Commit**

```bash
git add backend/src/api.rs backend/src/main.rs
git commit -m "feat: add GET /api/kpi endpoint"
```

---

## Task 3: Frontend types + API client

**Objective:** `web/src/lib/types.ts` gains KPI types; `web/src/lib/api.ts` gains `api.kpi()`.

**Files:**
- Modify: `web/src/lib/types.ts`
- Modify: `web/src/lib/api.ts`
- Test: `web/src/lib/kpi.test.ts` (new — for the pure helpers the page needs)

**Step 1: Write failing test — `web/src/lib/kpi.test.ts`**

```ts
import { describe, expect, it } from 'vitest';
import { denialRate, maxTrend, compact } from './kpi';

describe('kpi helpers', () => {
	it('denialRate: denied / (wasted+denied) * 100', () => {
		expect(denialRate(3, 1)).toBeCloseTo(25);
		expect(denialRate(0, 0)).toBe(0);
		expect(denialRate(0, 1)).toBe(100);
	});
	it('maxTrend: largest per-kind total for bar scaling', () => {
		const trend = [
			{ day: 'a', wasted: 2, denied: 1, affirmations: 3 },
			{ day: 'b', wasted: 5, denied: 0, affirmations: 0 }
		];
		expect(maxTrend(trend)).toBe(5);
	});
	it('compact: thousands separators', () => {
		expect(compact(0)).toBe('0');
		expect(compact(1234)).toBe('1,234');
		expect(compact(1234567)).toBe('1,234,567');
	});
});
```

**Step 2: Run to verify failure**

```bash
cd web && npx vitest run src/lib/kpi.test.ts
```

Expected: FAIL — module `./kpi` not found.

**Step 3: Create `web/src/lib/kpi.ts`**

```ts
// Pure helpers for the KPI dashboard page (no DOM, unit-testable).
export interface TrendPoint {
	day: string;
	wasted: number;
	denied: number;
	affirmations: number;
}

/** Of all loads committed, what % were denied. 0 when no loads. */
export function denialRate(wasted: number, denied: number): number {
	const total = wasted + denied;
	if (total === 0) return 0;
	return (denied / total) * 100;
}

/** Largest single-kind count across the trend — bar chart scale. */
export function maxTrend(trend: TrendPoint[]): number {
	return Math.max(1, ...trend.map((t) => Math.max(t.wasted, t.denied, t.affirmations)));
}

/** 1,234,567 formatting for big counters. */
export function compact(n: number): string {
	return n.toLocaleString('en-US');
}
```

**Step 4: Add KPI types to `web/src/lib/types.ts`**

```ts
export interface KpiTotals {
	total_wasted: number;
	total_denied: number;
	total_affirmations: number;
	total_users: number;
	active_24h: number;
	new_7d: number;
	currently_locked: number;
	total_lock_hours: number;
	denial_rate: number;
}

export interface KpiTrendPoint {
	day: string;
	wasted: number;
	denied: number;
	affirmations: number;
}

export interface KpiResponse {
	totals: KpiTotals;
	trend: KpiTrendPoint[];
	top_weekly: LeaderboardEntry[];
}
```

**Step 5: Add client method to `web/src/lib/api.ts`** (after `userOfTheDay`):

```ts
kpi: () => request<KpiResponse>('/kpi'),
```

(Add `KpiResponse` to the import list from `./types`.)

**Step 6: Run tests to verify pass**

```bash
npx vitest run src/lib/kpi.test.ts && npx vitest run
```

Expected: new test PASS, full frontend suite green (48 tests).

**Step 7: Commit**

```bash
git add web/src/lib/kpi.ts web/src/lib/kpi.test.ts web/src/lib/types.ts web/src/lib/api.ts
git commit -m "feat: add KPI types + api client + helpers"
```

---

## Task 4: Frontend page `/kpi`

**Objective:** Route renders KPI cards + 30-day stacked bar chart + top weekly board. Public.

**Files:**
- Create: `web/src/routes/kpi/+page.svelte`

**Step 1: Create route** (self-contained page, reuses global styles from `+layout.svelte` — `.card`, `.stat-grid`, `.stat-card`, `.board`, `.rank-top1..3`, `.empty`, `.skeleton`):

```svelte
<script lang="ts">
	import { api } from '$lib/api';
	import type { KpiResponse } from '$lib/types';
	import { timeAgo } from '$lib/api';
	import { onMount } from 'svelte';
	import { denialRate, maxTrend, compact } from '$lib/kpi';

	let data = $state<KpiResponse | null>(null);
	let error = $state<string | null>(null);

	onMount(async () => {
		try {
			data = await api.kpi();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load community stats';
		}
	});

	// stacked bar: wasted (red), denied (green), affirmations (dim). Scale = maxTrend.
	function barPct(v: number, max: number): string {
		return `${max === 0 ? 0 : Math.round((v / max) * 100)}%`;
	}
</script>

<svelte:head>
	<title>Community KPI — StreakForge</title>
</svelte:head>

<div class="container" style="max-width:860px;padding-top:28px;">
	<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 4px;">THE NUMBERS</h1>
	<p style="color:var(--text-dim);font-size:14px;margin:0 0 20px;">
		The board remembers. Live community stats — denial outranks everything.
	</p>

	{#if error}
		<div class="card empty" style="color:var(--red);">{error}</div>
	{:else if !data}
		<div class="skeleton" style="height:300px;"></div>
	{:else}
		{@const t = data.totals}
		<div class="stat-grid" style="margin-bottom:20px;">
			<div class="stat-card">
				<div class="label">Loads wasted</div>
				<div class="value red">{compact(t.total_wasted)}</div>
			</div>
			<div class="stat-card">
				<div class="label">Loads denied</div>
				<div class="value green">{compact(t.total_denied)}</div>
			</div>
			<div class="stat-card">
				<div class="label">Denial rate</div>
				<div class="value gold">{denialRate(t.total_wasted, t.total_denied).toFixed(1)}%</div>
			</div>
			<div class="stat-card">
				<div class="label">Affirmations</div>
				<div class="value">{compact(t.total_affirmations)}</div>
			</div>
			<div class="stat-card">
				<div class="label">Active · 24h</div>
				<div class="value">{t.active_24h}</div>
			</div>
			<div class="stat-card">
				<div class="label">New · 7d</div>
				<div class="value">{t.new_7d}</div>
			</div>
			<div class="stat-card">
				<div class="label">Currently locked</div>
				<div class="value">{t.currently_locked}</div>
			</div>
			<div class="stat-card">
				<div class="label">Lock hours · all-time</div>
				<div class="value">{Math.round(t.total_lock_hours).toLocaleString('en-US')}h</div>
			</div>
			<div class="stat-card">
				<div class="label">Whitebois</div>
				<div class="value">{compact(t.total_users)}</div>
			</div>
		</div>

		<h2 style="font-size:16px;letter-spacing:0.08em;margin:0 0 12px;">LAST 30 DAYS</h2>
		<div class="card" style="padding:18px;">
			{@const max = maxTrend(data.trend)}
			<div class="trend" style="display:flex;align-items:flex-end;gap:2px;height:120px;">
				{#each data.trend as p}
					<div class="trend-day" style="flex:1;display:flex;flex-direction:column;justify-content:flex-end;gap:1px;height:100%;" title="{p.day} · {p.wasted}💦 {p.denied}💧 {p.affirmations}✊">
						<div style="height:{barPct(p.affirmations, max)};background:var(--text-dim);opacity:.5;min-height:1px;"></div>
						<div style="height:{barPct(p.denied, max)};background:var(--green, #2ecc71);min-height:1px;"></div>
						<div style="height:{barPct(p.wasted, max)};background:var(--red);min-height:1px;"></div>
					</div>
				{/each}
			</div>
			<div style="display:flex;gap:16px;justify-content:center;margin-top:10px;font-size:12px;color:var(--text-dim);letter-spacing:.05em;">
				<span><span style="color:var(--red);">■</span> wasted</span>
				<span><span style="color:var(--green,#2ecc71);">■</span> denied</span>
				<span><span style="color:var(--text-dim);opacity:.6;">■</span> affirmations</span>
			</div>
		</div>

		<h2 style="font-size:16px;letter-spacing:0.08em;margin:24px 0 12px;">TOP THIS WEEK</h2>
		<div class="card" style="padding:0;overflow:hidden;">
			<table class="board">
				<thead>
					<tr>
						<th style="width:48px;">#</th>
						<th>WHITEBOI</th>
						<th style="text-align:right;">POINTS</th>
					</tr>
				</thead>
				<tbody>
					{#each data.top_weekly as e}
						<tr>
							<td class="rank" class:rank-top1={e.rank === 1} class:rank-top2={e.rank === 2} class:rank-top3={e.rank === 3}>{e.rank}</td>
							<td class="username">
								<a href={`/profile/${e.username}`}>{e.username}</a>
								{#if e.denial_count > 0}
									<span class="denial-badge" title="{e.denial_count} denials">💧{e.denial_count}</span>
								{/if}
							</td>
							<td class="count">{e.points}</td>
						</tr>
					{/each}
				</tbody>
			</table>
			{#if data.top_weekly.length === 0}
				<div class="empty" style="padding:24px;text-align:center;color:var(--text-dim);font-size:13px;">
					NO LOGS THIS WEEK — BE THE FIRST TO WASTE A LOAD
				</div>
			{/if}
		</div>

		<p style="color:var(--text-dim);font-size:12px;margin-top:10px;text-align:center;">
			Denial counts 10× a waste. Three affirmations = one wasted load.
		</p>
	{/if}
</div>
```

**Step 2: Verify build**

```bash
cd web && npx svelte-check && npm run build
```

Expected: no type errors, build succeeds.

**Step 3: Commit**

```bash
git add web/src/routes/kpi/+page.svelte
git commit -m "feat: add /kpi community dashboard page"
```

---

## Task 5: Nav link + docs

**Objective:** Discoverable link + repo docs current (all markdown must reflect reality).

**Files:**
- Modify: `web/src/routes/+layout.svelte` (footer — add "COMMUNITY KPI" link next to "Install the app")
- Modify: `README.md` (feature list: /kpi page)
- Modify: `STATUS.md` (checklist: KPI dashboard)
- Modify: `docs/OPERATIONS.md` (test counts: backend 18 → 19, frontend 47 → 48)

**Step 1: Footer link** — in `+layout.svelte` footer nav, add:

```svelte
<a href="/kpi" style="...">COMMUNITY KPI</a>
```

(match existing footer link styling — the "Install the app" anchor).

**Step 2: Docs updates**

- `README.md`: under features, add `- Community KPI dashboard (/kpi): wasted/denied totals, denial rate, active & new users, currently locked, lock hours, 30-day trend, top weekly`.
- `STATUS.md`: add a `KPI dashboard` row/checkbox (live) + migration 0008 note.
- `docs/OPERATIONS.md`: update test counts (backend 19, frontend 48) and add `GET /api/kpi` to the API surface list.

**Step 3: Commit**

```bash
git add web/src/routes/+layout.svelte README.md STATUS.md docs/OPERATIONS.md
git commit -m "docs: add KPI dashboard nav link + doc updates"
```

---

## Task 6: Full verification + deploy

**Objective:** Everything green locally, then live via the standard deploy.

**Step 1: Backend full suite**

```bash
cd backend && DATABASE_URL=postgres://streakforge:<pw>@127.0.0.1:5432/streakforge_test cargo test --test integration
```

Expected: 19 passed.

**Step 2: Frontend full suite + typecheck**

```bash
cd web && npx vitest run && npx svelte-check && npm run build
```

Expected: 48 passed, no errors.

**Step 3: Local smoke test**

```bash
# run backend locally, then:
curl -s http://127.0.0.1:8787/api/kpi | python3 -m json.tool | head -20
```

Expected: JSON with `totals`, `trend` (30 rows), `top_weekly`. Unknown API path still 404s (SPA-fallback guard untouched).

**Step 4: Deploy**

```bash
bash scripts/deploy.sh thinkcentre
```

**Step 5: Verify live** — the SPA-cache rule (skill pitfall): verify the **live bundle** contains the new page, not curl-status alone.

```bash
# 1) API works from the tunnel
curl -s https://streakforge.polarisocial.xyz/api/kpi | head -c 200
# 2) live bundle contains /kpi route string (cache-bust note: immutable hashed assets)
curl -s https://streakforge.polarisocial.xyz/_app/immutable/start/start.js | grep -c kpi || true
# better: fetch the live page and grep the hydration chunk for "THE NUMBERS"
curl -s https://streakforge.polarisocial.xyz/kpi | grep -o "THE NUMBERS" | head -1
# 3) content-type sanity for the page route is SPA html — expected 200
curl -s -o /dev/null -w "%{http_code} %{content_type}\n" https://streakforge.polarisocial.xyz/kpi
```

Expected: `/api/kpi` JSON; bundle/page contains the KPI strings; `/kpi` returns `200 text/html`.

**Step 6: No follow-up commit needed unless the verify step found a bug** — if so, fix + test + commit before declaring done.

---

## Acceptance criteria

- [ ] `GET /api/kpi` returns totals, 30-day trend, top-weekly (public, no auth)
- [ ] `/kpi` page renders 9 stat cards, stacked bar chart, top-weekly table, empty states
- [ ] Footer link present; mobile BottomNav unchanged
- [ ] Backend tests: 19/19 green (new `kpi_totals_and_trend`)
- [ ] Frontend tests: 48/48 green (new `kpi.test.ts`); `svelte-check` clean
- [ ] Repo docs current: README, STATUS, docs/OPERATIONS
- [ ] Deployed via `scripts/deploy.sh thinkcentre`; live `/api/kpi` + `/kpi` verified against the LIVE bundle
- [ ] No `load$` copy anywhere (maintainer rule); theme preserved (denial encouraged)

## Risks / notes

- **`numeric` → `f64` decode**: `kpi_totals()` returns `numeric` for `total_lock_hours`/`denial_rate`. sqlx decodes `numeric` to `f64` fine for these magnitudes; if the compile complains, cast in SQL (`::float8`) or decode `BigDecimal`. Prefer SQL cast (`total_lock_hours::float8`, `denial_rate::float8`) — zero-Rust change.
- **`time::Date` serialization**: avoided entirely — `kpi_trend` returns `day` as `text` via `to_char`.
- **Migration is additive**: new functions only; no `drop function` of existing ones. Safe to apply on the live DB via deploy script.
- **NFS wedges**: dev repo is on `/personal`; if `cargo build` or `npm run build` throws EIO/SIGBUS mid-task, recover per `streakforge-development` skill (remount /personal; rebuild, don't trust the interrupted artifact).
