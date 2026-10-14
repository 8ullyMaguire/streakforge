<script lang="ts">
	import { api, ApiRequestError } from '$lib/api';
	import type { Stats } from '$lib/types';
	import { onMount } from 'svelte';
	import LogButton from '$lib/components/LogButton.svelte';
	import StreakCalendar from '$lib/components/StreakCalendar.svelte';
	import { budgetLabel } from '$lib/countdown';

	let stats = $state<Stats | null>(null);
	let error = $state<string | null>(null);
	// When the current stats payload landed locally. LogButton anchors its
	// countdown to this so the remaining seconds tick down from the server's
	// number instead of trusting the browser clock to agree with it.
	let receivedAt = $state(Date.now());

	// Ticks slowly so the budget line's "resets in 4h 12m" stays honest without
	// a reload. 30s, not 1s: the label is in minutes, and a per-second interval
	// on a page that also runs a per-second countdown is work nobody can see.
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
		<p style="color:var(--text-dim);font-size:14px;margin:0 0 20px;">
			Loads wasted are a budget of {stats.daily_limit} a day (UTC) — miss every one and
			the streak resets. A denial and a waste are mutually exclusive.
		</p>

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
				Decay + lifetime. A score with no explanation reads as a bug when it
				drops, so say plainly WHY it is what it is: 30 days of silence wipes
				the score, and the record survives separately.
			-->
			{#if stats.devotion.decayed}
				<div
					style="margin-top:10px;padding:9px 11px;border-radius:8px;background:rgba(227,28,35,0.10);border:1px solid rgba(227,28,35,0.35);font-size:12px;line-height:1.5;color:var(--red-soft);"
				>
					Score wiped — 30 days without a log. Your record of
					<span style="color:var(--text);font-weight:600;">{stats.devotion.lifetime_total}</span>
					lifetime log{stats.devotion.lifetime_total === 1 ? '' : 's'} is intact, but the score resets with
					the next log you make.
				</div>
			{:else if stats.devotion.days_until_decay !== null && stats.devotion.days_until_decay <= 10}
				<!--
					Item 16: the decay used to be silent. A user found out by
					returning to a zero, which reads as the site having lost their
					score rather than as a rule they can act on. Ten days is late
					enough to be about the real state and early enough to log once.
				-->
				<div
					style="margin-top:10px;padding:9px 11px;border-radius:8px;background:rgba(212,175,55,0.10);border:1px solid rgba(212,175,55,0.35);font-size:12px;line-height:1.5;color:var(--gold);"
				>
					{#if stats.devotion.days_until_decay === 0}
						Your score resets today — it has been 30 days.
					{:else}
						Your score resets in {stats.devotion.days_until_decay}
						{stats.devotion.days_until_decay === 1 ? 'day' : 'days'}.
					{/if}
					One log keeps the era alive.
				</div>
			{:else}
				<div style="margin-top:8px;font-size:12px;color:var(--text-dim);">
					lifetime <span style="color:var(--text);">{stats.devotion.lifetime_total}</span> logs
					<span style="opacity:0.75;">· score resets if you go 30 days without logging</span>
				</div>
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
				<span>{budgetLabel(stats.remaining_today, stats.daily_limit, stats.next_allowed_in_secs, now - receivedAt)}</span>
				{#if stats.blocked_by_exclusivity}
					<span style="color:var(--gold);">the other kind already ran today</span>
				{/if}
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
