<script lang="ts">
	import { api } from '$lib/api';
	import type { Leaderboard, LeaderboardPeriod, UserOfTheDay } from '$lib/types';
	import { onMount } from 'svelte';
	import { timeAgo } from '$lib/api';

	type Tab = LeaderboardPeriod;
	const tabs: { id: Tab; label: string; hint: string }[] = [
		{ id: 'daily', label: 'DAILY', hint: 'rolling 24 hours' },
		{ id: 'weekly', label: 'WEEKLY', hint: 'rolling 7 days' },
		{ id: 'monthly', label: 'MONTHLY', hint: 'rolling 30 days' },
		{ id: 'alltime', label: 'ALL-TIME', hint: 'every log, ever' }
	];

	// MONTHLY is the default view.
	const DEFAULT_TAB: Tab = 'monthly';

	let active = $state<Tab>(DEFAULT_TAB);
	let boards = $state<Partial<Record<Tab, Leaderboard>>>({});
	let uotd = $state<UserOfTheDay | null>(null);
	let loading = $state(false);

	onMount(() => {
		loadTab(DEFAULT_TAB);
		api
			.userOfTheDay()
			.then((u) => (uotd = u))
			.catch(() => {});
	});

	async function loadTab(t: Tab) {
		if (boards[t]) return;
		loading = true;
		try {
			boards[t] = await api.leaderboard(t);
		} catch {
			// leave the tab empty; the skeleton resolves to the empty state
		} finally {
			loading = false;
		}
	}

	// A score of 8.75 should not render as "8.7500000001" or "9".
	function fmtScore(n: number): string {
		return Number.isInteger(n) ? String(n) : n.toFixed(2).replace(/0+$/, '').replace(/\.$/, '');
	}

	function activeKindLabel(kind: string): string {
		if (kind === 'habit') return 'WLW';
		if (kind === 'denial') return 'WLD';
		return '—';
	}

	// [streakFrom, streakTo|null, multiplier] — null means "and up".
	const TIERS: { from: number; to: number | null; mult: number }[] = [
		{ from: 1, to: 6, mult: 1.0 },
		{ from: 7, to: 13, mult: 1.25 },
		{ from: 14, to: 29, mult: 1.5 },
		{ from: 30, to: 59, mult: 2.0 },
		{ from: 60, to: 99, mult: 2.5 },
		{ from: 100, to: null, mult: 3.0 }
	];
</script>

<svelte:head>
	<title>Leaderboard — StreakForge</title>
</svelte:head>

<div class="container" style="max-width:860px;padding-top:28px;">
	<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 4px;">
		WHITEBOIS WHO KNOW THEIR PLACE
	</h1>
	<p style="color:var(--text-dim);font-size:12px;letter-spacing:0.06em;margin:0 0 20px;">
		WHITEBOI DEVOTION INDEX — SCORE = (WLW + WLD + GAME BONUS) × MULTIPLIER
	</p>

	{#if uotd}
		<div class="uotd" style="margin-bottom:24px;">
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
	{/if}

	<div class="tabs">
		{#each tabs as t}
			<button class="tab" class:active={active === t.id} onclick={() => { active = t.id; loadTab(t.id); }}>
				{t.label}
			</button>
		{/each}
	</div>
	<p style="color:var(--text-dim);font-size:11px;letter-spacing:0.06em;margin:6px 0 10px;text-align:center;">
		{tabs.find((t) => t.id === active)?.hint}
	</p>

	{#if !boards[active]}
		<div class="skeleton" style="height:300px;"></div>
	{:else}
		<div class="card" style="padding:0;overflow:hidden;">
			<table class="board">
				<thead>
					<tr>
						<th style="width:44px;">#</th>
						<th>WHITEBOI</th>
						<th style="text-align:right;">SCORE</th>
						<th style="text-align:right;">BREAKDOWN</th>
						<th style="text-align:right;">STREAK</th>
						<th style="text-align:right;">LAST</th>
					</tr>
				</thead>
				<tbody>
					{#each boards[active]!.entries as e}
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
							<td style="text-align:right;color:var(--text-dim);font-size:11px;white-space:nowrap;">
								{e.wlw}💦 {e.wld}💧 {e.game_bonus}✊
								{#if e.multiplier !== 1}
									<span style="color:var(--gold);">×{fmtScore(e.multiplier)}</span>
								{/if}
							</td>
							<td style="text-align:right;font-size:12px;white-space:nowrap;">
								{e.streak}d
								<span style="color:var(--text-dim);font-size:11px;">{activeKindLabel(e.active_kind)}</span>
							</td>
							<td style="text-align:right;color:var(--text-dim);font-size:12px;">
								{#if e.last_log_at}{timeAgo(e.last_log_at)}{/if}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
			{#if boards[active]!.entries.length === 0}
				<div style="padding:24px;text-align:center;color:var(--text-dim);font-size:13px;letter-spacing:0.05em;">
					NO SCORE IN THIS WINDOW — BE THE FIRST TO LOG
				</div>
			{/if}
		</div>

		<details style="margin-top:14px;">
			<summary style="cursor:pointer;color:var(--text-dim);font-size:12px;letter-spacing:0.08em;">
				SCORING RULES
			</summary>
			<div style="margin-top:10px;padding:14px;background:var(--bg-card);border:1px solid var(--border);border-radius:6px;">
				<p style="margin:0 0 8px;font-size:13px;color:var(--text);">
					<b>Score = ((WLW + WLD + Game Bonus) × Score Multiplier) − Racism Penalty</b>
				</p>
				<p style="margin:0 0 8px;font-size:12px;color:var(--text-dim);">
					WLWs and WLDs are mutually exclusive — recording one breaks the other's streak. The
					multiplier comes from your active exclusive streak. Submit once a day (UTC) to grow
					it; miss a day and it resets.
				</p>
				<table style="width:100%;font-size:12px;border-collapse:collapse;">
					<thead>
						<tr style="color:var(--text-dim);text-align:left;">
							<th style="padding:3px 0;">STREAK</th>
							<th style="padding:3px 0;">MULTIPLIER</th>
						</tr>
					</thead>
					<tbody>
						{#each TIERS as tier}
							<tr>
								<td style="padding:3px 0;color:var(--text);">
									{tier.to === null ? `${tier.from}+ days` : `${tier.from}–${tier.to} days`}
								</td>
								<td style="padding:3px 0;color:var(--gold);font-family:var(--font-mono);">
									{fmtScore(tier.mult)}×
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
				<p style="margin:10px 0 0;font-size:11px;color:var(--text-dim);">
					Game Bonus = 1 point per 3 affirmations. Racism Penalty is currently always 0 — the
					term is in the formula, but StreakForge has no reporting data source for it yet.
				</p>
			</div>
		</details>
	{/if}
</div>
