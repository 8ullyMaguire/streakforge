<script lang="ts">
	import { api, ApiRequestError } from '$lib/api';
	import type { Stats } from '$lib/types';
	import { onMount } from 'svelte';
	import LogButton from '$lib/components/LogButton.svelte';
	import StreakCalendar from '$lib/components/StreakCalendar.svelte';
	import { budgetLabel, formatCountdown } from '$lib/countdown';
	import { decayNotice, decayStyle } from '$lib/decay';
	import { pushToast } from '$lib/toasts.svelte';
	import { todayLine } from '$lib/today';
	import type { TodayCounts } from '$lib/today';

	let stats = $state<Stats | null>(null);
	let error = $state<string | null>(null);
	// When the current stats payload landed locally. LogButton anchors its
	// countdown to this so the remaining seconds tick down from the server's
	// number instead of trusting the browser clock to agree with it.
	let receivedAt = $state(Date.now());
	// The decay sentence, from a module with its own tests: the copy is the only
	// explanation of the 30-day rule a user ever sees, so it is asserted rather
	// than written inline where nothing can reach it.
	let decay = $derived(stats ? decayNotice(stats.devotion) : null);

	// Ticks slowly so the budget line's "resets in 4h 12m" stays honest without
	// a reload. 30s, not 1s: the label is in minutes, and a per-second interval
	// on a page that also runs a per-second countdown is work nobody can see.
	// Item 10: what the COMMUNITY has done today, which is a different number from
	// the user's own. Fetched separately and tolerated separately -- the landing
	// page learned that one failing endpoint must not blank the page, and this
	// line is decoration next to the numbers above it.
	let today = $state<TodayCounts | null>(null);

	let now = $state(Date.now());
	$effect(() => {
		const t = setInterval(() => (now = Date.now()), 30_000);
		return () => clearInterval(t);
	});

	function last30Days(s: Stats): { date: string; count: number }[] {
		const byDay = new Map<string, number>();
		for (const l of s.recent_logs) {
			const d = new Date(l.logged_at);
			const key = d.toISOString().slice(0, 10);
			byDay.set(key, (byDay.get(key) ?? 0) + 1);
		}
		const out: { date: string; count: number }[] = [];
		const now = new Date();
		for (let i = 29; i >= 0; i--) {
			const d = new Date(Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate() - i));
			const key = d.toISOString().slice(0, 10);
			out.push({ date: key, count: byDay.get(key) ?? 0 });
		}
		return out;
	}

	onMount(async () => {
		// Independent of the stats call: this is a public read, and a failure
		// leaves the line absent rather than taking the dashboard with it.
		fetch('/api/today')
			.then((r) => (r.ok ? r.json() : null))
			.then((t) => (today = t))
			.catch(() => (today = null));

		try {
			stats = await api.stats();
			receivedAt = Date.now();
		} catch (e) {
			if (e instanceof ApiRequestError && e.status === 401) {
				window.location.href = '/login';
				return;
			}
			error = e instanceof Error ? e.message : 'Failed to load stats';
		}
	});

	// The yesterday window. One per filing day, and only into a day that holds
	// nothing — the server checks both, so this is a straight POST and a redraw.
	let catchingUp = $state(false);
	async function catchUp(kind: 'habit' | 'denial') {
		if (catchingUp) return;
		catchingUp = true;
		try {
			const res = await api.logHabit(undefined, kind, undefined, 'yesterday');
			handleLogged(res.stats);
			pushToast(kind === 'denial' ? 'Denial filed for yesterday.' : 'Load filed for yesterday.', 'success');
		} catch (e) {
			pushToast(e instanceof Error ? e.message : 'Could not file that.', 'error');
		} finally {
			catchingUp = false;
		}
	}

	function handleLogged(s: Stats) {
		stats = s;
		// Fresh payload: the server just reset its own remaining-seconds, so
		// re-baseline rather than letting the old anchor run to zero early.
		receivedAt = Date.now();
	}

	function calendarClass(day: Date): string {
		// 0 = none, 1 = streak active, 2 = streak broken (missed yesterday)
		return '1';
	}

	// A score of 8.75 should not render as "8.7500000001" or "9".
	function fmtScore(n: number): string {
		return Number.isInteger(n) ? String(n) : n.toFixed(2).replace(/0+$/, '').replace(/\.$/, '');
	}
</script>

<svelte:head>
	<title>Dashboard — StreakForge</title>
</svelte:head>

<div class="container" style="max-width:760px;padding-top:28px;">
	{#if error}
		<div class="card empty" style="color:var(--red);">{error}</div>
	{:else if !stats}
		<div class="skeleton" style="height:300px;"></div>
	{:else}
		<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 4px;">
			KNOW YOUR PLACE
		</h1>
		<p style="color:var(--text-dim);font-size:14px;margin:0 0 10px;">
			Loads wasted are a budget of {stats.daily_limit} a day (UTC) — miss every one and
			the streak resets. A waste spends the day's denial; a denial does not spend a waste.
		</p>

		<!--
			STREAK AT RISK. The multiplier only pays consecutive days, and a member who
			has already earned one has no way of noticing that today's silence will cost
			it at midnight. This is the cheapest retention mechanic on the site: no
			transport, no new table, just the clock the server already sends.
		-->
		{#if stats.at_risk.at_risk}
			<div
				class="card"
				style="margin:0 0 18px;padding:12px 14px;border-color:rgba(212,175,55,0.45);font-size:13px;line-height:1.55;"
			>
				<span style="color:var(--gold);font-weight:600;">YOUR ×{fmtScore(stats.at_risk.multiplier)} IS AT RISK.</span>
				<span style="color:var(--text-dim);">
					{stats.at_risk.streak} days of showing up, and today is not one of them yet —
					it dies at 00:00 UTC. {formatCountdown(stats.at_risk.secs_left)} left.
				</span>
			</div>
		{/if}

		<!--
			Item 10: the community's day, not the user's. This is the only number on
			this page that is about other people, and it is deliberately a live read
			of the same count the KPI page publishes rather than a stored copy.
			The sentence is a pure function in $lib/today so today.test.ts can assert
			the empty state as well as the busy one.
		-->
		{#if today}
			<div style="font-size:13px;margin:0 0 20px;">
				<a href="/kpi" style="color:var(--gold);text-decoration:none;">
					{todayLine(today)}
				</a>
			</div>
		{:else}
			<div style="height:19px;margin:0 0 20px;"></div>
		{/if}

		<!-- Whiteboi Devotion Index -->
		<div class="card" style="margin-bottom:20px;">
			<div style="display:flex;justify-content:space-between;align-items:baseline;gap:12px;flex-wrap:wrap;">
				<div>
					<div style="font-size:11px;letter-spacing:0.1em;color:var(--text-dim);">
						WHITEBOI DEVOTION INDEX
					</div>
					<div
						style="font-family:var(--font-mono);font-size:38px;line-height:1.1;color:var(--gold);"
					>
						{fmtScore(stats.devotion.score)}
					</div>
				</div>
				<div style="text-align:right;font-size:12px;color:var(--text-dim);">
					<div>
						{#if stats.devotion.active_kind === 'denial'}WLD{:else if stats.devotion.active_kind === 'habit'}WLW{:else}—{/if}
						streak <span style="color:var(--text);">{stats.devotion.active_streak}d</span>
					</div>
					{#if stats.devotion.multiplier !== 1}
						<div>multiplier <span style="color:var(--gold);">×{fmtScore(stats.devotion.multiplier)}</span></div>
					{/if}
				</div>
			</div>
			<div
				style="margin-top:12px;padding-top:12px;border-top:1px solid var(--border);display:flex;gap:18px;flex-wrap:wrap;font-size:12px;color:var(--text-dim);"
			>
				<span>💦 {stats.devotion.wlw} WLW</span>
				<span>💧 {stats.devotion.wld} WLD</span>
				<span>✊ {stats.devotion.game_bonus} bonus</span>
				<span style="color:var(--text-dim);">
					= ({stats.devotion.wlw} + {stats.devotion.wld} + {stats.devotion.game_bonus}) × {fmtScore(stats.devotion.multiplier)}
				</span>
			</div>

			<!--
				Decay + lifetime, in three states: wiped, counting down (ten days or
				fewer), and quiet. A score with no explanation reads as a bug when it
				drops, so say plainly WHY it is what it is: 30 days of silence wipes
				the score, and the record survives separately. The sentences live in
				$lib/decay.ts, where `decay.test.ts` asserts them.
			-->
			{#if decay}
				<div style={decayStyle(decay.kind)}>{decay.text}</div>
			{/if}
		</div>

		<div class="stat-grid" style="margin-bottom:20px;">
			<div class="stat-card">
				<div class="label">Today</div>
				<div class="value green">{stats.today_count}</div>
			</div>
			<div class="stat-card">
				<div class="label">Current streak</div>
				<div class="value red">{stats.current_streak}<span style="font-size:14px;color:var(--text-dim);">d</span></div>
			</div>
			<div class="stat-card">
				<div class="label">Longest</div>
				<div class="value gold">{stats.longest_streak}<span style="font-size:14px;color:var(--text-dim);">d</span></div>
			</div>
			<div class="stat-card">
				<div class="label">This week</div>
				<div class="value">{stats.week_count}</div>
			</div>
			<div class="stat-card">
				<div class="label">All-time</div>
				<div class="value">{stats.alltime_count}</div>
			</div>
		</div>

		<!--
			YESTERDAY IS EMPTY. A member who remembers the day after it closes had no
			path but a broken streak. The window is deliberately narrow and says so:
			one catch-up per filing day, and only into a day with nothing in it.
		-->
		{#if stats.catch_up.available}
			<div class="card" style="margin:0 0 18px;padding:14px 16px;">
				<div style="font-size:11px;letter-spacing:0.12em;color:var(--gold);">YESTERDAY IS EMPTY</div>
				<div style="font-size:13px;color:var(--text-dim);margin-top:6px;line-height:1.55;">
					Days are UTC and it is already the next one. One catch-up is allowed per day, and it
					fills a gap — it cannot add to a day you already kept.
				</div>
				<div style="display:flex;gap:8px;margin-top:10px;flex-wrap:wrap;">
					<button class="btn btn-red" style="padding:5px 14px;font-size:12px;" onclick={() => catchUp('habit')} disabled={catchingUp}>
						WASTE FOR YESTERDAY
					</button>
					<button class="btn btn-ghost" style="padding:5px 14px;font-size:12px;" onclick={() => catchUp('denial')} disabled={catchingUp}>
						DENIAL FOR YESTERDAY
					</button>
				</div>
			</div>
		{:else if stats.catch_up.filed_today}
			<div style="margin:0 0 18px;font-size:12px;color:var(--text-dim);">
				Catch-up used today — one a day, and it is spent.
			</div>
		{/if}

		<!--
			Each kind's own streak. One combined number hid the one-way rule that the
			server enforces (a waste spends the day's denial; a denial never spends a
			waste), and a rule nobody can see reads as a bug when it bites.
		-->
		<div class="stat-grid" style="margin-bottom:20px;">
			<div class="stat-card">
				<div class="label">WLW streak</div>
				<div class="value red">{stats.kind_streaks.habit}<span style="font-size:14px;color:var(--text-dim);">d</span></div>
			</div>
			<div class="stat-card">
				<div class="label">WLD streak</div>
				<div class="value gold">{stats.kind_streaks.denial}<span style="font-size:14px;color:var(--text-dim);">d</span></div>
			</div>
			<div class="stat-card">
				<div class="label">Drill streak</div>
				<div class="value green">{stats.kind_streaks.affirmation}<span style="font-size:14px;color:var(--text-dim);">d</span></div>
			</div>
		</div>

		<!--
			The multiplier tiers, as progress rather than a mystery: the thresholds are
			the ones `devotion_multiplier` actually pays, sent by the server so the page
			cannot promise a number the SQL will not give.
		-->
		<div class="card" style="margin-bottom:24px;padding:14px 16px;">
			<div style="font-size:11px;letter-spacing:0.1em;color:var(--text-dim);margin-bottom:8px;">
				MULTIPLIER MILESTONES
			</div>
			<div style="display:flex;gap:8px;flex-wrap:wrap;">
				{#each stats.milestones as m (m.days)}
					<div
						style="flex:1;min-width:96px;padding:8px 10px;border:1px solid {m.reached ? 'rgba(40,160,90,0.5)' : 'var(--border)'};border-radius:6px;"
					>
						<div style="font-size:11px;color:var(--text-dim);">{m.days} days</div>
						<div style="font-family:var(--font-mono);font-size:16px;color:{m.reached ? 'var(--green)' : 'var(--text-dim)'};">
							×{fmtScore(m.multiplier)}
						</div>
					</div>
				{/each}
			</div>
			{#if stats.at_risk.next_tier_in !== null}
				<div style="font-size:12px;color:var(--text-dim);margin-top:8px;">
					{stats.at_risk.next_tier_in} more day{stats.at_risk.next_tier_in === 1 ? '' : 's'} of showing up to the next tier.
				</div>
			{/if}
		</div>

		<div class="card" style="margin-bottom:24px;">
			<!--
				Item 15: the remaining budget and the reset clock in ONE line, in
				the place where a user decides whether to press the button. Both
				numbers come from the server — `remaining_today` is what it will
				enforce, and the countdown is anchored to when the payload landed
				(`receivedAt`) so the display cannot drift against the server's
				clock. The label itself is a pure function in $lib/countdown so it
				can be tested with fake timers instead of a rendered page.
			-->
			<div
				style="display:flex;justify-content:space-between;gap:12px;flex-wrap:wrap;font-size:12px;letter-spacing:0.04em;color:var(--text-dim);margin-bottom:12px;"
			>
				<span>{budgetLabel(stats.remaining_today, stats.daily_limit, stats.secs_until_reset, now - receivedAt)}</span>
			</div>
			<LogButton stats={() => stats!} onLogged={handleLogged} {receivedAt} />
		</div>

		<h2 style="font-size:16px;letter-spacing:0.08em;margin:0 0 12px;">LAST 30 DAYS</h2>
		<div class="card" style="margin-bottom:24px;">
			<StreakCalendar days={last30Days(stats)} />
		</div>

		<h2 style="font-size:16px;letter-spacing:0.08em;margin:0 0 12px;">RECENT LOGS</h2>
		<div class="card" style="padding:4px 18px;">
			{#if stats.recent_logs.length === 0}
				<div class="empty">
					<div class="big">🔥</div>
					No logs yet — hit the button!
					<div style="font-size:12px;margin-top:10px;">
						New here? The <a href="/glossary">glossary</a> defines WLW, WLD, a denial,
						a drill rep, and what the score is made of.
					</div>
				</div>
			{:else}
				{#each stats.recent_logs as item}
					<div class="feed-item">
						<div>
							<span style="color:var(--text-dim);font-size:13px;">
								{new Date(item.logged_at).toLocaleString()}
							</span>
							{#if item.note}
								<div class="note">“{item.note}”</div>
							{/if}
						</div>
					</div>
				{/each}
			{/if}
		</div>
	{/if}
</div>
