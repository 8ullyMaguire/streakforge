<script lang="ts">
	import { api } from '$lib/api';
	import type { Leaderboard, LeaderboardPeriod, UserOfTheDay } from '$lib/types';
	import { onMount } from 'svelte';
	import { timeAgo } from '$lib/api';

	type Tab = LeaderboardPeriod;
	const tabs: { id: Tab; label: string }[] = [
		{ id: 'daily', label: 'DAILY' },
		{ id: 'weekly', label: 'WEEKLY' },
		{ id: 'alltime', label: 'ALL-TIME' }
	];

	let active = $state<Tab>('daily');
	let boards = $state<Partial<Record<Tab, Leaderboard>>>({});
	let uotd = $state<UserOfTheDay | null>(null);
	let loading = $state(false);

	onMount(() => {
		loadTab('daily');
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
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Leaderboard — StreakForge</title>
</svelte:head>

<div class="container" style="max-width:720px;padding-top:28px;">
	<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 20px;">
		WHITEBOIS WHO KNOW THEIR PLACE
	</h1>

	{#if uotd}
		<div class="uotd" style="margin-bottom:24px;">
			<div class="crown">👑</div>
			<div>WHITEBOI OF THE DAY</div>
			<div class="handle">{uotd.username}</div>
			<div class="count">{uotd.points} points · 24h</div>
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

	{#if !boards[active]}
		<div class="skeleton" style="height:300px;"></div>
	{:else}
		<div class="card" style="padding:0;overflow:hidden;">
			<table class="board">
				<thead>
					<tr>
						<th style="width:48px;">#</th>
						<th>WHITEBOI</th>
						<th style="text-align:right;">POINTS</th>
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
							<td class="count">
								{e.points}
								<span style="color:var(--text-dim);font-size:11px;margin-left:4px;">
									({e.waste_count}💦 {e.denial_count}💧 {e.affirmation_count}✊)
								</span>
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
					NO LOGS IN THIS WINDOW — BE THE FIRST TO WASTE A LOAD
				</div>
			{/if}
		</div>
		<p style="color:var(--text-dim);font-size:12px;margin-top:10px;text-align:center;">
			Denial counts 10× a waste. Three affirmations = one wasted load. Points = waste + denial×10 + affs÷3.
		</p>
	{/if}
</div>
