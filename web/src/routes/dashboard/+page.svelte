<script lang="ts">
	import { api, ApiRequestError } from '$lib/api';
	import type { Stats } from '$lib/types';
	import { onMount } from 'svelte';
	import LogButton from '$lib/components/LogButton.svelte';
	import StreakCalendar from '$lib/components/StreakCalendar.svelte';

	let stats = $state<Stats | null>(null);
	let error = $state<string | null>(null);

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
	}

	function calendarClass(day: Date): string {
		// 0 = none, 1 = streak active, 2 = streak broken (missed yesterday)
		return '1';
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
			Waste only when permitted. One per hour. Five per day. Denial outranks everything.
		</p>

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
			<LogButton stats={() => stats!} onLogged={handleLogged} />
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
