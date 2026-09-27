<script lang="ts">
	import { api } from '$lib/api';
	import type { Leaderboard, UserOfTheDay, FeedItem } from '$lib/types';
	import { timeAgo, formatCount } from '$lib/api';
	import { onMount } from 'svelte';
	import Marquee from '$lib/components/Marquee.svelte';

	let total = $state<number | null>(null);
	let denied = $state<number | null>(null);
	let monthly = $state<Leaderboard | null>(null);
	let uotd = $state<UserOfTheDay | null>(null);
	let recentFeed = $state<FeedItem[]>([]);
	let loaded = $state(false);
	let signedIn = $state(false);

	onMount(async () => {
		// allSettled, NOT all: the comment below used to claim partial failure was
		// tolerated, but Promise.all rejects on the FIRST failure, so a single
		// failing endpoint blanked the counters, the board and the feed at once.
		// Each panel is independent; one being down must not take the page with it.
		const [uotdRes, dailyRes, feedRes, totalRes] = await Promise.allSettled([
			api.userOfTheDay(),
			api.leaderboard('monthly'),
			api.feed('0'),
			fetch('/api/total').then((r) => r.json())
		]);

		if (uotdRes.status === 'fulfilled' && uotdRes.value) uotd = uotdRes.value;
		if (dailyRes.status === 'fulfilled') monthly = dailyRes.value;
		if (feedRes.status === 'fulfilled') recentFeed = feedRes.value.items.slice(0, 8);
		if (totalRes.status === 'fulfilled' && totalRes.value) {
			total = totalRes.value.total;
			denied = totalRes.value.denied ?? 0;
		}

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
			<a href="/dashboard" class="btn btn-red btn-lg">WASTE A LOAD</a>
		{:else}
			<a href="/login" class="btn btn-red btn-lg">KNOW YOUR PLACE</a>
		{/if}
		<a href="/leaderboard" class="btn btn-ghost btn-lg">VIEW THE BOARD</a>
	</div>
	<p style="color:var(--text-dim);font-size:13px;margin-top:14px;">
		{#if signedIn}
			Waste only when permitted. The board remembers.
		{:else}
			Create an account. Own your submission. Deny harder.
		{/if}
	</p>
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
			Score = (WLW + WLD + Game Bonus) × streak multiplier. The board rewards
			<span style="color:var(--text);">devotion</span>, not volume.
		</p>
	{/if}

	{#if recentFeed.length > 0}
		<h2 style="font-size:18px;letter-spacing:0.08em;margin:28px 0 12px;">CONFESSION FEED</h2>
		<div class="card" style="padding:4px 18px;">
			{#each recentFeed as item}
				<div class="feed-item">
					<div>
						<span class="username" style="font-weight:600;">
							<a href={`/profile/${item.username}`}>{item.username}</a>
						</span>
						{#if item.note}
							<div class="note">“{item.note}”</div>
						{/if}
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
