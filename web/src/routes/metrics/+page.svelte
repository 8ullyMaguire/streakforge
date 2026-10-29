<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import { formatDuration, formatDurationShort } from '$lib/duration';
	import type { BoardsResponse, TimeToCumResponse } from '$lib/types';

	// METRICS — the numbers the site keeps but does not score.
	//
	// "Time to cum" is the owner's definition: how long the session ran, start to
	// release. Both averages are shown because they answer different questions: the
	// pooled one is "what does a log look like", the per-person one gives every
	// member one vote. Hiding the second would let one heavy logger define the site.

	let ttc = $state<TimeToCumResponse | null>(null);
	let boards = $state<BoardsResponse | null>(null);
	let error = $state<string | null>(null);

	onMount(async () => {
		const [a, b] = await Promise.allSettled([api.timeToCum(), api.boards()]);
		if (a.status === 'fulfilled') ttc = a.value;
		else error = a.reason instanceof Error ? a.reason.message : 'Could not load metrics.';
		if (b.status === 'fulfilled') boards = b.value;
	});
</script>

<svelte:head>
	<title>Metrics — StreakForge</title>
</svelte:head>

<div class="container" style="max-width:720px;padding-top:28px;">
	<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 8px;">THE NUMBERS</h1>
	<p style="color:var(--text-dim);font-size:13px;margin:0 0 20px;line-height:1.6;">
		Nothing here is scored. Time to cum is self-reported: from the moment you start to the
		moment you release, per log. Report it when logging — one click records the load, the
		number is optional and can be added for fifteen minutes after.
		<b>These averages include only members who turned sharing on</b> — it is off by
		default, and it is the one setting on the site that starts off. You always see your own
		numbers below, sharing or not.
	</p>

	{#if error}
		<div class="card" style="padding:12px 14px;margin-bottom:16px;color:var(--red-soft);font-size:13px;">
			{error}
		</div>
	{/if}

	{#if ttc}
		<h2 style="font-size:16px;letter-spacing:0.08em;margin:0 0 12px;">TIME TO CUM</h2>
		<div class="stat-grid" style="margin-bottom:16px;">
			<div class="stat-card">
				<div class="label">Average (every log)</div>
				<div class="value gold">{formatDuration(ttc.aggregate.avg_secs)}</div>
			</div>
			<div class="stat-card">
				<div class="label">Median</div>
				<div class="value">{formatDuration(ttc.aggregate.median_secs)}</div>
			</div>
			<div class="stat-card">
				<div class="label">Average per person</div>
				<div class="value green">{formatDuration(ttc.aggregate.avg_of_user_avgs_secs)}</div>
			</div>
			<div class="stat-card">
				<div class="label">Reports</div>
				<div class="value">{ttc.aggregate.samples}<span style="font-size:14px;color:var(--text-dim);"> · {ttc.aggregate.reporters} whitebois</span></div>
			</div>
		</div>

		{#if ttc.aggregate.reporters === 0}
			<div class="card" style="padding:16px;margin-bottom:20px;">
				<div style="font-size:13px;line-height:1.6;">
					<b>Nobody shares yet.</b> The averages above are pooled from members who opted in,
					and no one has — so they are zero because there is nothing to average, not because
					the sessions are instant.
					{#if ttc.sharing === false}
						<a href="/settings" style="color:var(--gold);">Turn sharing on in settings</a> if you
						want yours counted.
					{:else if ttc.sharing === null}
						<a href="/login" style="color:var(--gold);">Sign in</a> to share yours.
					{:else}
						You share yours; you are the first in the pool — one member is not an average, so
						nothing is printed until someone joins you.
					{/if}
				</div>
			</div>
		{:else if ttc.sharing === false}
			<div style="font-size:12px;color:var(--text-dim);margin:-6px 0 18px;">
				You are not sharing your durations — <a href="/settings" style="color:var(--gold);">change that in settings</a>.
				{ttc.aggregate.reporters} member{ttc.aggregate.reporters === 1 ? '' : 's'} do.
			</div>
		{/if}

		{#if ttc.mine}
			<div class="card" style="padding:16px;margin-bottom:20px;">
				<div style="font-size:12px;letter-spacing:0.12em;color:var(--text-dim);margin-bottom:8px;">YOURS</div>
				{#if ttc.mine.samples === 0}
					<div style="font-size:13px;color:var(--text-dim);">
						No durations reported yet. Log a load, then say how long it took.
					</div>
				{:else}
					<div style="display:flex;gap:22px;flex-wrap:wrap;font-size:14px;">
						<span>avg <b class="gold">{formatDuration(ttc.mine.avg_secs)}</b></span>
						<span>median <b>{formatDuration(ttc.mine.median_secs)}</b></span>
						<span>fastest <b class="green">{formatDuration(ttc.mine.min_secs)}</b></span>
						<span>slowest <b class="red">{formatDuration(ttc.mine.max_secs)}</b></span>
						<span>reports <b>{ttc.mine.samples}</b></span>
					</div>
				{/if}
			</div>
		{/if}

		<h2 style="font-size:16px;letter-spacing:0.08em;margin:0 0 12px;">LONGEST EDGES</h2>
		<div class="card" style="padding:4px 18px;margin-bottom:24px;">
			{#if ttc.board.length === 0}
				<div class="empty">
					<div class="big">⏱️</div>
					{#if ttc.aggregate.reporters === 0}
						Nobody shares their durations yet, so there is no board to rank.
					{:else}
						Nobody who shares has five reported sessions yet.
					{/if}
					<div style="font-size:12px;margin-top:10px;">
						The board needs five reports per person so one four-hour outlier is not a
						ranking strategy, and it only ever ranks members who turned sharing on.
					</div>
				</div>
			{:else}
				{#each ttc.board as row, i (row.username)}
					<div style="display:flex;align-items:center;gap:12px;padding:10px 0;border-bottom:1px solid var(--border);">
						<span style="width:22px;color:var(--text-dim);font-family:var(--font-mono);">{i + 1}</span>
						<a href={`/profile/${row.username}`} style="flex:1;font-weight:600;">{row.username}</a>
						<span style="font-family:var(--font-mono);color:var(--gold);">{formatDuration(row.avg_secs)}</span>
						<span style="font-size:12px;color:var(--text-dim);">{row.samples} reports</span>
					</div>
				{/each}
			{/if}
		</div>
	{/if}

	{#if boards}
		<h2 style="font-size:16px;letter-spacing:0.08em;margin:0 0 12px;">THE OTHER BOARDS</h2>
		<p style="color:var(--text-dim);font-size:12px;margin:0 0 12px;">
			Last 30 days. The main board ranks the score; these rank the behaviours the score is
			made of.
		</p>
		<div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(210px,1fr));gap:14px;">
			<div class="card" style="padding:14px;">
				<div style="font-size:12px;letter-spacing:0.12em;color:var(--red);margin-bottom:8px;">MOST DENIED</div>
				{#if boards.most_denied.length === 0}
					<div style="font-size:12px;color:var(--text-dim);">Nothing yet.</div>
				{:else}
					{#each boards.most_denied as row, i (row.username)}
						<div style="display:flex;justify-content:space-between;font-size:13px;padding:3px 0;">
							<span>{i + 1}. <a href={`/profile/${row.username}`}>{row.username}</a></span>
							<span style="color:var(--gold);">{row.denials}</span>
						</div>
					{/each}
				{/if}
			</div>
			<div class="card" style="padding:14px;">
				<div style="font-size:12px;letter-spacing:0.12em;color:var(--green);margin-bottom:8px;">CONSISTENCY</div>
				{#if boards.consistency.length === 0}
					<div style="font-size:12px;color:var(--text-dim);">Nothing yet.</div>
				{:else}
					{#each boards.consistency as row, i (row.username)}
						<div style="display:flex;justify-content:space-between;font-size:13px;padding:3px 0;">
							<span>{i + 1}. <a href={`/profile/${row.username}`}>{row.username}</a></span>
							<span style="color:var(--green);">{Math.round(row.pct)}%</span>
						</div>
					{/each}
				{/if}
			</div>
			<div class="card" style="padding:14px;">
				<div style="font-size:12px;letter-spacing:0.12em;color:var(--gold);margin-bottom:8px;">LOCK STREAK</div>
				{#if boards.locks.length === 0}
					<div style="font-size:12px;color:var(--text-dim);">Nothing yet.</div>
				{:else}
					{#each boards.locks as row, i (row.username)}
						<div style="display:flex;justify-content:space-between;font-size:13px;padding:3px 0;">
							<span>{i + 1}. <a href={`/profile/${row.username}`}>{row.username}</a></span>
							<span style="color:var(--gold);">{row.current_streak}d</span>
						</div>
					{/each}
				{/if}
			</div>
		</div>
		<p style="color:var(--text-dim);font-size:12px;margin-top:14px;">
			The lifetime counters live on the landing page; these boards count the same rows by a
			narrower window.
		</p>
	{/if}
</div>
