<script lang="ts">
	import { api } from '$lib/api';
	import type { KpiResponse } from '$lib/types';
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

	// Stacked bar scale. maxTrend floors at 1 so an all-zero site still divides.
	function barPct(v: number, max: number): string {
		return `${max === 0 ? 0 : Math.round((v / max) * 100)}%`;
	}

	// Copied from the leaderboard pages rather than invented: a score of 8.75
	// should not render as "8.7500000001" or "9".
	function fmtScore(n: number): string {
		return Number.isInteger(n) ? String(n) : n.toFixed(2).replace(/0+$/, '').replace(/\.$/, '');
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
		{@const max = maxTrend(data.trend)}
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
			<div class="trend" style="display:flex;align-items:flex-end;gap:2px;height:120px;">
				{#each data.trend as p}
					<div class="trend-day" style="flex:1;display:flex;flex-direction:column;justify-content:flex-end;gap:1px;height:100%;" title="{p.day} · {p.wasted} wasted {p.denied} denied {p.affirmations} affirmed">
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
						<th style="text-align:right;">SCORE</th>
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
							<td class="count">{fmtScore(e.score)}</td>
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
