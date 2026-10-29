<script lang="ts">
	import { api, ApiRequestError } from '$lib/api';
	import { pushToast } from '$lib/toasts.svelte';
	import { TTC_CHIPS, parseDurationInput } from '$lib/duration';
	import type { Leaderboard, UserOfTheDay, PublicFeedItem, TodayCounts } from '$lib/types';
	import { timeAgo, formatCount } from '$lib/api';
	import { onMount } from 'svelte';
	import Marquee from '$lib/components/Marquee.svelte';

	let total = $state<number | null>(null);
	let denied = $state<number | null>(null);
	let monthly = $state<Leaderboard | null>(null);
	let uotd = $state<UserOfTheDay | null>(null);
	let recentFeed = $state<PublicFeedItem[]>([]);
	// Item 10: who was denied today, next to a total that only ever grows. The
	// lifetime counters cannot say whether the place is alive RIGHT NOW.
	let today = $state<TodayCounts | null>(null);
	let loaded = $state(false);
	let signedIn = $state(false);

	onMount(async () => {
		// allSettled, NOT all: the comment below used to claim partial failure was
		// tolerated, but Promise.all rejects on the FIRST failure, so a single
		// failing endpoint blanked the counters, the board and the feed at once.
		// Each panel is independent; one being down must not take the page with it.
		const [uotdRes, dailyRes, feedRes, totalRes, todayRes] = await Promise.allSettled([
			api.userOfTheDay(),
			api.leaderboard('monthly'),
			api.feed('0'),
			fetch('/api/total').then((r) => r.json()),
			fetch('/api/today').then((r) => r.json())
		]);

		if (uotdRes.status === 'fulfilled' && uotdRes.value) uotd = uotdRes.value;
		if (dailyRes.status === 'fulfilled') monthly = dailyRes.value;
		if (feedRes.status === 'fulfilled') recentFeed = feedRes.value.items.slice(0, 8);
		if (totalRes.status === 'fulfilled' && totalRes.value) {
			total = totalRes.value.total;
			denied = totalRes.value.denied ?? 0;
		}
		if (todayRes.status === 'fulfilled' && todayRes.value) today = todayRes.value;

		// check auth to decide the primary CTA
		api
			.me()
			.then((u) => {
				signedIn = !!u;
			})
			.catch(() => {
				signedIn = false;
			})
			.finally(() => {
				loaded = true;
			});
	});

	// render total as wlw-style colored digits: first 3 red, last 3 green, middle black
	function digitClass(i: number, len: number) {
		if (i < 3) return 'digit-red';
		if (i >= len - 3) return 'digit-green';
		return 'digit-black';
	}

	// A score of 8.75 should not render as "8.7500000001" or "9".
	function fmtScore(n: number): string {
		return Number.isInteger(n) ? String(n) : n.toFixed(2).replace(/0+$/, '').replace(/\.$/, '');
	}

	// ---- ONE CLICK (owner, 2026-10-29) ------------------------------------
	// "logging waste/denial from landing page should be a single click rather than
	// multiple". So: one POST, no confirm page, no drill gate, no navigation. The
	// two things a person might still want to say — how long it took, and "that was
	// a mistake" — are offered AFTER the fact, inline, because both are optional and
	// neither should stand between a click and its record.
	let logging = $state(false);
	let recorded = $state<{ id: number; kind: 'habit' | 'denial' } | null>(null);
	let logError = $state<string | null>(null);
	let ttcOpen = $state(false);
	let ttcCustom = $state('');
	let ttcSaved = $state<number | null>(null);
	let undoing = $state(false);

	async function logNow(kind: 'habit' | 'denial') {
		if (logging) return;
		logging = true;
		logError = null;
		// Deliberately NOT clearing `recorded` here. A refused second attempt (a
		// denial after a waste, say) used to wipe the undo card belonging to the log
		// that DID land — the one case where the follow-up matters most. It is
		// replaced on success and left alone on failure.
		try {
			const res = await api.logHabit(undefined, kind);
			// The id of the row just written is the newest one in the caller's own
			// recent logs — no extra endpoint, and it is theirs by construction.
			recorded = { id: Number(res.stats.recent_logs[0]?.id ?? 0), kind };
			ttcOpen = false;
			ttcSaved = null;
			pushToast(kind === 'denial' ? 'Denial recorded.' : 'Load recorded.', 'success');
			// The counters are public and only ever move forward: bump locally so the
			// page does not need a reload to agree with what the server just did.
			if (kind === 'denial') denied = (denied ?? 0) + 1;
			else total = (total ?? 0) + 1;
		} catch (e) {
			// 429 here is a rule, not a failure: the server's sentence names the clock.
			logError = e instanceof ApiRequestError || e instanceof Error ? e.message : 'Could not record that.';
			pushToast(logError ?? 'Could not record that.', 'error');
		} finally {
			logging = false;
		}
	}

	async function saveTtc(secs: number | null) {
		if (!recorded || recorded.id === 0) return;
		try {
			await api.patchLog(recorded.id, secs);
			ttcSaved = secs;
			ttcOpen = false;
			pushToast(secs === null ? 'Duration cleared.' : 'Duration saved.', 'success');
		} catch (e) {
			pushToast(e instanceof Error ? e.message : 'Could not save that.', 'error');
		}
	}

	function submitCustom() {
		const secs = parseDurationInput(ttcCustom);
		if (secs === null) {
			pushToast('That is not a duration — try 12:30, 20m or 1h.', 'error');
			return;
		}
		void saveTtc(secs);
	}

	async function undo() {
		if (!recorded || recorded.id === 0) return;
		undoing = true;
		try {
			await api.deleteLog(recorded.id);
			if (recorded.kind === 'denial') denied = Math.max(0, (denied ?? 1) - 1);
			else total = Math.max(0, (total ?? 1) - 1);
			recorded = null;
			ttcSaved = null;
			pushToast('Undone. As if it never happened.', 'success');
		} catch (e) {
			pushToast(e instanceof Error ? e.message : 'Could not undo that.', 'error');
		} finally {
			undoing = false;
		}
	}
</script>

<svelte:head>
	<title>StreakForge — Embrace Defeat. The Future Is Black.</title>
</svelte:head>

<section class="hero">
	<div class="tagline">EMBRACE DEFEAT. THE FUTURE IS BLACK.</div>
	<h1>StreakForge</h1>
	<p class="sub">
		Whitebois don't just track habits. They deny. They lock. They waste when permitted.
		And the board remembers who knows their place.
	</p>
	{#if total !== null}
		<div class="counter counter-digits" aria-label="Loads wasted {total}">
			{#each String(total).padStart(9, '0').split('') as ch, i (i)}
				<span class={digitClass(i, 9)}>{ch}</span>
			{/each}
		</div>
		<p style="color:var(--text-dim);font-size:13px;letter-spacing:0.2em;text-transform:uppercase;margin:4px 0 8px;">
			loads wasted
		</p>
	{/if}
	{#if denied !== null && denied > 0}
		<div class="counter counter-digits counter-denied" aria-label="Loads denied {denied}">
			{#each String(denied).padStart(9, '0').split('') as ch, i (i)}
				<span class={digitClass(i, 9)}>{ch}</span>
			{/each}
		</div>
		<p style="color:var(--text-dim);font-size:13px;letter-spacing:0.2em;text-transform:uppercase;margin:4px 0 8px;">
			loads denied
		</p>
	{/if}

	<div class="cta-row">
		{#if signedIn}
			<button class="btn btn-red btn-lg" onclick={() => logNow('habit')} disabled={logging}>
				{logging ? 'RECORDING…' : 'WASTE A LOAD'}
			</button>
			<button class="btn btn-ghost btn-lg" onclick={() => logNow('denial')} disabled={logging}>
				I DENIED
			</button>
		{:else}
			<a href="/login?next=/dashboard" class="btn btn-red btn-lg">KNOW YOUR PLACE</a>
			<a href="/leaderboard" class="btn btn-ghost btn-lg">VIEW THE BOARD</a>
		{/if}
	</div>

	{#if signedIn}
		<p style="color:var(--text-dim);font-size:13px;margin-top:14px;">
			One click records it. A waste spends the day's denial; a denial never spends a waste.
		</p>
	{:else}
		<p style="color:var(--text-dim);font-size:13px;margin-top:14px;">
			Create an account. Own your submission. Deny harder.
		</p>
	{/if}

	{#if logError}
		<div
			class="card"
			style="max-width:520px;margin:16px auto 0;padding:12px 14px;font-size:13px;color:var(--red-soft);border-color:rgba(200,30,30,0.35);text-align:left;"
		>
			{logError}
		</div>
	{/if}

	{#if recorded}
		<div class="card" style="max-width:520px;margin:16px auto 0;padding:14px 16px;text-align:left;">
			<div style="font-size:12px;letter-spacing:0.14em;color:var(--green);">
				RECORDED — {recorded.kind === 'denial' ? 'LOAD DENIED' : 'LOAD WASTED'}
			</div>
			{#if !ttcOpen && ttcSaved === null}
				<div style="margin-top:10px;font-size:13px;color:var(--text-dim);">
					How long did it take, start to release?
					<div style="display:flex;flex-wrap:wrap;gap:8px;margin-top:8px;">
						{#each TTC_CHIPS as chip (chip.secs)}
							<button class="btn btn-ghost" style="padding:4px 12px;font-size:12px;" onclick={() => saveTtc(chip.secs)}>
								{chip.label}
							</button>
						{/each}
						<button class="btn btn-ghost" style="padding:4px 12px;font-size:12px;" onclick={() => (ttcOpen = true)}>
							other…
						</button>
					</div>
				</div>
			{:else if ttcOpen}
				<div style="margin-top:10px;display:flex;gap:8px;flex-wrap:wrap;">
					<input
						placeholder="12:30 or 20m"
						bind:value={ttcCustom}
						style="flex:1;min-width:120px;"
						onkeydown={(e) => e.key === 'Enter' && submitCustom()}
					/>
					<button class="btn btn-red" style="padding:4px 12px;font-size:12px;" onclick={submitCustom}>SAVE</button>
				</div>
			{:else}
				<div style="margin-top:10px;font-size:13px;color:var(--text-dim);">
					Duration saved{ttcSaved ? ` (${ttcSaved}s)` : ''}.
				</div>
			{/if}
			<div style="margin-top:12px;">
				<button class="btn btn-ghost" style="padding:4px 12px;font-size:12px;" onclick={undo} disabled={undoing}>
					{undoing ? 'UNDOING…' : 'UNDO (15 min)'}
				</button>
			</div>
		</div>
	{/if}
</section>

{#if recentFeed.length > 0}
	<Marquee
		items={recentFeed.slice(0, 10).map((i) => ({ username: i.username, time: timeAgo(i.logged_at) }))}
	/>
{/if}

{#if uotd}
	<div class="container" style="max-width:640px;margin-bottom:28px;">
		<div class="uotd">
			<div class="crown">👑</div>
			<div>WHITEBOI OF THE DAY</div>
			<div class="handle">{uotd.username}</div>
			<div class="count">
				{uotd.points}
				{uotd.points === 1 ? 'point' : 'points'} · 24h
			</div>
			<div class="sub">
				{#if uotd.display_name}{uotd.display_name} · {/if}
				first log {timeAgo(uotd.first_log_at)} · {uotd.alltime_count} all-time
			</div>
		</div>
	</div>
{/if}

<div class="container">
	{#if monthly}
		<h2 style="font-size:18px;letter-spacing:0.08em;margin:20px 0 4px;">
			WHITEBOIS WHO KNOW THEIR PLACE
		</h2>
		<p style="color:var(--text-dim);font-size:11px;letter-spacing:0.06em;margin:0 0 12px;text-align:center;">
			WHITEBOI DEVOTION INDEX · MONTHLY (ROLLING 30 DAYS)
		</p>
		<div class="card" style="padding:0;overflow:hidden;">
			<table class="board">
				<thead>
					<tr>
						<th style="width:44px;">#</th>
						<th>WHITEBOI</th>
						<th style="text-align:right;">SCORE</th>
						<th style="text-align:right;">STREAK</th>
					</tr>
				</thead>
				<tbody>
					{#each monthly.entries.slice(0, 25) as e}
						<tr>
							<td class="rank" class:rank-top1={e.rank === 1} class:rank-top2={e.rank === 2} class:rank-top3={e.rank === 3}>
								{e.rank}
							</td>
							<td class="username">
								<a href={`/profile/${e.username}`}>{e.username}</a>
								{#if e.denial_count > 0}
									<span class="denial-badge" title="{e.denial_count} denials">💧{e.denial_count}</span>
								{/if}
							</td>
							<td class="count">{fmtScore(e.score)}</td>
							<td style="text-align:right;font-size:12px;white-space:nowrap;">
								{e.streak}d
								{#if e.multiplier !== 1}
									<span style="color:var(--gold);">×{fmtScore(e.multiplier)}</span>
								{/if}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<p style="color:var(--text-dim);font-size:12px;margin-top:10px;text-align:center;">
			Score = (WLW + WLD + Game Bonus) × multiplier, which follows days of participation. The board rewards
			<span style="color:var(--text);">devotion</span>, not volume.
		</p>
	{/if}

	{#if today && today.loggers_today > 0}
		<!-- Item 10: one line, under the lifetime totals it qualifies. -->
		<p style="color:var(--text-dim);font-size:12px;text-align:center;margin-top:14px;">
			<span style="color:var(--text);">{today.denials_today}</span>
			{today.denials_today === 1 ? 'denial' : 'denials'} today ·
			<span style="color:var(--text);">{today.wastes_today}</span>
			{today.wastes_today === 1 ? 'load' : 'loads'} wasted ·
			<span style="color:var(--text);">{today.loggers_today}</span>
			{today.loggers_today === 1 ? 'whiteboi' : 'whitebois'} logged in
		</p>
	{/if}

	{#if recentFeed.length > 0}
		<h2 style="font-size:18px;letter-spacing:0.08em;margin:28px 0 6px;">RECENT ACTIVITY</h2>
		<p style="color:var(--text-dim);font-size:12px;margin:0 0 12px;">
			Confessions are public by default — turn yours off in <a href="/settings">settings</a>.
		</p>
		<div class="card" style="padding:4px 18px;">
			{#each recentFeed as item}
				<div class="feed-item">
					<div>
						<span class="username" style="font-weight:600;">
							<a href={`/profile/${item.username}`}>{item.username}</a>
						</span>
					</div>
					<div style="flex:1;"></div>
					<span class="time">{timeAgo(item.logged_at)}</span>
				</div>
			{/each}
		</div>
	{/if}
</div>

{#if !loaded}
	<div class="container">
		<div class="skeleton" style="height:220px;margin-top:24px;"></div>
	</div>
{/if}

<style>
	.counter-denied {
		margin-top: 10px;
		opacity: 0.85;
	}
</style>
