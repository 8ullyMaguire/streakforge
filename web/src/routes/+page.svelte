<script lang="ts">
	import { api } from '$lib/api';
	import type { Leaderboard, UserOfTheDay, FeedItem } from '$lib/types';
	import { timeAgo, formatCount } from '$lib/api';
	import { onMount } from 'svelte';
	import Marquee from '$lib/components/Marquee.svelte';

	let total = $state<number | null>(null);
	let daily = $state<Leaderboard | null>(null);
	let uotd = $state<UserOfTheDay | null>(null);
	let recentFeed = $state<FeedItem[]>([]);
	let loaded = $state(false);

	onMount(async () => {
		try {
			const [uotdRes, dailyRes, feedRes, totalRes] = await Promise.all([
				api.userOfTheDay(),
				api.leaderboard('daily'),
				api.feed('0'),
				fetch('/api/total').then((r) => r.json())
			]);
			uotd = uotdRes;
			daily = dailyRes;
			recentFeed = feedRes.items.slice(0, 8);
			total = totalRes.total;
		} catch {
			// tolerate partial failure — page still renders
		}
		loaded = true;
	});

	// render total as wlw-style colored digits: green for last 3, black for the rest
	function digitClass(i: number, len: number) {
		return i >= len - 3 ? 'digit-green' : 'digit-black';
	}
</script>

<svelte:head>
	<title>StreakForge — Consistency is Power</title>
</svelte:head>

<section class="hero">
	<div class="tagline">EMBRACE THE STREAK. THE FUTURE IS CONSISTENT.</div>
	<h1>StreakForge</h1>
	<p class="sub">
		Log your daily completions. Build unbreakable streaks. Climb the leaderboard.
		Own your consistency.
	</p>
	{#if total !== null}
		<div class="counter counter-digits" aria-label="Total completions {total}">
			{#each String(total).padStart(8, '0').split('') as ch, i (i)}
				<span class={digitClass(i, 8)}>{ch}</span>
			{/each}
		</div>
	{/if}

	<div class="cta-row">
		<a href="/login" class="btn btn-red btn-lg">START LOGGING</a>
		<a href="/leaderboard" class="btn btn-ghost btn-lg">VIEW LEADERBOARD</a>
	</div>
	<p style="color:var(--text-dim);font-size:13px;margin-top:14px;">
		Create an account. Own your submission.
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
			<div>USER OF THE DAY</div>
			<div class="handle">{uotd.username}</div>
			<div class="count">{uotd.count} completions today</div>
			<div class="sub">
				{#if uotd.display_name}{uotd.display_name} · {/if}
				first log {timeAgo(uotd.first_log_at)} · {uotd.alltime_count} all-time
			</div>
		</div>
	</div>
{/if}

<div class="container">
	{#if daily}
		<h2 style="font-size:18px;letter-spacing:0.08em;margin:20px 0 12px;">
			TOP FORGERS TODAY
		</h2>
		<div class="card" style="padding:0;overflow:hidden;">
			<table class="board">
				<thead>
					<tr>
						<th style="width:48px;">#</th>
						<th>FORGER</th>
						<th style="text-align:right;">COUNT</th>
					</tr>
				</thead>
				<tbody>
					{#each daily.entries.slice(0, 25) as e}
						<tr>
							<td class="rank" class:rank-top1={e.rank === 1} class:rank-top2={e.rank === 2} class:rank-top3={e.rank === 3}>
								{e.rank}
							</td>
							<td class="username">
								<a href={`/profile/${e.username}`}>{e.username}</a>
							</td>
							<td class="count">{e.count}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}

	{#if recentFeed.length > 0}
		<h2 style="font-size:18px;letter-spacing:0.08em;margin:28px 0 12px;">RECENT ACTIVITY</h2>
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
