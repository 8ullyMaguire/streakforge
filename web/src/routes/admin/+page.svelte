<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiRequestError } from '$lib/api';
	import { formatDurationShort } from '$lib/duration';
	import type { AdminNumbers } from '$lib/types';

	// THE NUMBERS THAT DECIDE THINGS — trust level 4 only (idea #123).
	//
	// Everything here is a count over a window, and every definition is printed beside the
	// table it belongs to. That is not decoration: "retention" and "active" are the two words
	// most likely to be read as something kinder than they are, and the server ships its own
	// definitions so this page cannot paraphrase them.
	//
	// No member's name appears anywhere on this page, deliberately. A product dashboard that
	// names people becomes a surveillance tool; the questions here are "is this growing" and
	// "do anonymous visitors come back", neither of which needs a person.

	let data = $state<AdminNumbers | null>(null);
	let error = $state<string | null>(null);
	let forbidden = $state(false);

	onMount(async () => {
		try {
			data = await api.adminNumbers();
		} catch (e) {
			if (e instanceof ApiRequestError && e.status === 403) forbidden = true;
			else error = e instanceof Error ? e.message : 'Could not load the numbers.';
		}
	});

	function pct(part: number | null, whole: number): string {
		if (part === null) return '—';
		if (whole === 0) return '—';
		return `${Math.round((part / whole) * 100)}%`;
	}

	function bars(weeks: { active_members: number }[]): number {
		return Math.max(1, ...weeks.map((w) => w.active_members));
	}

	const maxActive = $derived(data ? bars(data.weekly) : 1);
</script>

<svelte:head>
	<title>Numbers — StreakForge</title>
</svelte:head>

<div class="container" style="max-width:860px;padding-top:28px;">
	<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 8px;">PRODUCT NUMBERS</h1>
	<p style="color:var(--text-dim);font-size:13px;margin:0 0 20px;line-height:1.6;">
		Aggregates only — no member is named here, and nothing on this page is scored. Read it,
		decide something, close it.
	</p>

	{#if forbidden}
		<div class="card" style="padding:18px;font-size:13px;line-height:1.6;">
			<b>This page belongs to the site's leader.</b>
			<span style="color:var(--text-dim);">
				The numbers are aggregates, but they answer the questions an owner asks — growth,
				conversion, retention — and nobody else needs them. If something here matters to you,
				ask a leader.
			</span>
		</div>
	{:else if error}
		<div class="card" style="padding:12px 14px;color:var(--red-soft);font-size:13px;">{error}</div>
	{:else if data}
		<!-- This week -->
		<div class="stat-grid" style="margin-bottom:8px;">
			<div class="stat-card">
				<div class="label">Active members (7d)</div>
				<div class="value green">{data.overview.active_7d}</div>
			</div>
			<div class="stat-card">
				<div class="label">Active today</div>
				<div class="value">{data.overview.active_1d}</div>
			</div>
			<div class="stat-card">
				<div class="label">New members (7d)</div>
				<div class="value gold">{data.overview.new_7d}</div>
			</div>
			<div class="stat-card">
				<div class="label">Median logs / active</div>
				<div class="value">{data.overview.median_logs_per_active_7d.toFixed(1)}</div>
			</div>
			<div class="stat-card">
				<div class="label">Logs (7d)</div>
				<div class="value">{data.overview.logs_7d}</div>
			</div>
			<div class="stat-card">
				<div class="label">Members, all time</div>
				<div class="value">{data.overview.members}</div>
			</div>
		</div>

		<!-- The funnel -->
		<h2 style="font-size:16px;letter-spacing:0.08em;margin:24px 0 10px;">GUEST → ACCOUNT</h2>
		<div class="card" style="padding:16px;margin-bottom:8px;">
			<div style="display:flex;gap:26px;flex-wrap:wrap;font-size:14px;align-items:baseline;">
				<span>{data.overview.guests_created} guests ever</span>
				<span style="color:var(--green);">{data.overview.guests_converted} registered</span>
				<span style="font-family:var(--font-mono);font-size:22px;color:var(--gold);">
					{data.overview.conversion_rate === null
						? '—'
						: `${Math.round(data.overview.conversion_rate * 100)}%`}
				</span>
			</div>
			<div style="font-size:12px;color:var(--text-dim);margin-top:10px;line-height:1.6;">
				A converted guest is counted in the week it first appeared, so the rate answers "of the
				people who tried anonymously, how many came back with a name" — not "how many guests
				converted this week".
			</div>
		</div>

		<!-- Weekly -->
		<h2 style="font-size:16px;letter-spacing:0.08em;margin:24px 0 10px;">BY WEEK</h2>
		<div class="card" style="padding:6px 14px;margin-bottom:8px;overflow-x:auto;">
			<div style="display:grid;grid-template-columns:96px 1fr 58px 58px 66px;gap:8px;font-size:11px;letter-spacing:0.08em;color:var(--text-dim);padding:8px 0;border-bottom:1px solid var(--border);">
				<span>WEEK</span><span>ACTIVE MEMBERS</span><span>NEW</span><span>LOGS</span
				><span>GUESTS</span>
			</div>
			{#each data.weekly as w (w.week_start)}
				<div style="display:grid;grid-template-columns:96px 1fr 58px 58px 66px;gap:8px;align-items:center;font-size:13px;padding:6px 0;border-bottom:1px solid var(--border);">
					<span style="font-family:var(--font-mono);color:var(--text-dim);">{w.week_start}</span>
					<span style="display:flex;align-items:center;gap:8px;">
						<span style="flex:1;height:10px;background:var(--border);border-radius:2px;overflow:hidden;min-width:40px;">
							<span
								style="display:block;height:100%;background:var(--green);width:{Math.round(
									(w.active_members / maxActive) * 100
								)}%;"
							></span>
						</span>
						<span style="font-family:var(--font-mono);width:28px;text-align:right;">
							{w.active_members}
						</span>
					</span>
					<span style="font-family:var(--font-mono);color:var(--gold);">{w.new_members}</span>
					<span style="font-family:var(--font-mono);">{w.logs}</span>
					<span style="font-family:var(--font-mono);color:var(--text-dim);">
						{w.guests}{w.guests > 0 ? `→${w.guests_converted}` : ''}
					</span>
				</div>
			{/each}
		</div>
		<p style="color:var(--text-dim);font-size:11px;margin:0 0 8px;">
			ISO weeks, Monday 00:00 UTC. The last row is the current week and is incomplete.
		</p>

		<!-- Retention -->
		<h2 style="font-size:16px;letter-spacing:0.08em;margin:24px 0 10px;">RETENTION</h2>
		<div class="card" style="padding:6px 14px;margin-bottom:8px;overflow-x:auto;">
			<div style="display:grid;grid-template-columns:96px 54px repeat(4, 1fr);gap:8px;font-size:11px;letter-spacing:0.08em;color:var(--text-dim);padding:8px 0;border-bottom:1px solid var(--border);">
				<span>COHORT</span><span>SIZE</span><span>W1</span><span>W2</span><span>W3</span
				><span>W4</span>
			</div>
			{#each data.retention as c (c.cohort_start)}
				<div style="display:grid;grid-template-columns:96px 54px repeat(4, 1fr);gap:8px;align-items:center;font-size:13px;padding:6px 0;border-bottom:1px solid var(--border);">
					<span style="font-family:var(--font-mono);color:var(--text-dim);">{c.cohort_start}</span>
					<span style="font-family:var(--font-mono);">{c.cohort_size}</span>
					{#each [c.w1, c.w2, c.w3, c.w4] as v}
						<span style="font-family:var(--font-mono);color:{v === null ? 'var(--text-dim)' : 'var(--text)'};">
							{#if v === null}
								—
							{:else}
								{v}
								<span style="color:var(--text-dim);font-size:11px;">{pct(v, c.cohort_size)}</span>
							{/if}
						</span>
					{/each}
				</div>
			{/each}
		</div>
		<p style="color:var(--text-dim);font-size:11px;margin:0 0 8px;">
			Share of a signup week still logging in the n-th week after it. A dash means that week has
			not happened yet — not that everyone left.
		</p>

		<!-- The rest of the counts -->
		<div class="card" style="padding:14px;margin-top:18px;font-size:13px;color:var(--text-dim);display:flex;gap:22px;flex-wrap:wrap;">
			<span>Verified cage: <b style="color:var(--green);">{formatDurationShort(data.overview.verified_cage_secs)}</b></span>
			<span>Lines in the deck: <b style="color:var(--text);">{data.overview.affirmations_live}</b></span>
			<span>Logs today: <b style="color:var(--text);">{data.overview.logs_1d}</b></span>
		</div>

		<details style="margin-top:18px;font-size:12px;color:var(--text-dim);">
			<summary style="cursor:pointer;letter-spacing:0.06em;">HOW THESE ARE COUNTED</summary>
			<ul style="margin:8px 0 0;padding-left:18px;line-height:1.7;">
				{#each data.notes as note (note)}
					<li>{note}</li>
				{/each}
			</ul>
		</details>

		<p style="color:var(--text-dim);font-size:11px;margin-top:16px;">
			Read at {new Date(data.generated_at).toLocaleString()}.
		</p>
	{/if}
</div>
