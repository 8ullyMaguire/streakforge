<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiRequestError, timeAgo } from '$lib/api';
	import { pushToast } from '$lib/toasts.svelte';
	import { formatDuration, formatDurationShort } from '$lib/duration';
	import type { CageCheck, CageResponse } from '$lib/types';

	// CAGE — the check, the seal, and the peers who vouch.
	//
	// Nothing is uploaded here. A check is a timestamped claim with a code the site
	// issued, and the hours only count once other members confirm it; that is what
	// "verified by users themselves" means in practice, so the page says so rather
	// than implying a photo was reviewed by a machine.

	let data = $state<CageResponse | null>(null);
	let queue = $state<CageCheck[]>([]);
	let note = $state('');
	let busy = $state(false);
	let error = $state<string | null>(null);
	let attesting = $state<number | null>(null);

	async function load() {
		try {
			const [cage, q] = await Promise.all([api.cage(), api.cageQueue()]);
			data = cage;
			queue = q;
			error = null;
		} catch (e) {
			error =
				e instanceof ApiRequestError || e instanceof Error ? e.message : 'Could not load the cage.';
		}
	}

	onMount(load);

	async function check() {
		if (busy) return;
		busy = true;
		error = null;
		try {
			await api.cageCheck(note.trim() || undefined);
			note = '';
			pushToast('Checked. Now someone has to vouch for it.');
			await load();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Could not record the check.';
			pushToast(error ?? 'Could not record the check.', 'error');
		} finally {
			busy = false;
		}
	}

	async function attest(id: number, verdict: 'confirm' | 'dispute') {
		attesting = id;
		try {
			const updated = await api.attest(id, verdict);
			pushToast(
				updated.status === 'verified'
					? 'That check is verified now.'
					: updated.status === 'disputed'
						? 'Disputed — its hours are out until it is restored.'
						: 'Noted. One more voice and it counts.'
			);
			await load();
		} catch (e) {
			pushToast(e instanceof Error ? e.message : 'Could not attest that.', 'error');
		} finally {
			attesting = null;
		}
	}

	// A lock nobody has checked for a day is the abandoned-timer case; asking is
	// cheaper than pretending the hours are still flowing.
	let stale = $derived.by(() => {
		if (!data?.lock.locked) return false;
		const last = data?.metrics.last_check_at;
		if (!last) return true;
		const hours = (Date.now() - new Date(last).getTime()) / 3_600_000;
		return hours >= 24;
	});
</script>

<svelte:head>
	<title>Cage — StreakForge</title>
</svelte:head>

<div class="container" style="max-width:720px;padding-top:28px;">
	<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 8px;">THE CAGE</h1>
	<p style="color:var(--text-dim);font-size:13px;margin:0 0 20px;line-height:1.6;">
		A check is your word plus a seal the site gives you. Write that code where only a caged
		whiteboi could — a card next to the lock — and peers confirm it. Hours count only once
		they do. Nothing is uploaded, nothing is scanned.
	</p>

	{#if error}
		<div class="card" style="padding:12px 14px;margin-bottom:16px;color:var(--red-soft);font-size:13px;">
			{error}
		</div>
	{/if}

	{#if data}
		<div class="card" style="padding:16px;margin-bottom:16px;">
			<div style="display:flex;justify-content:space-between;align-items:baseline;gap:12px;flex-wrap:wrap;">
				<div style="font-size:14px;">
					{#if data.lock.locked}
						<span style="color:var(--gold);">LOCKED</span>
						<span style="color:var(--text-dim);font-size:12px;">
							· {timeAgo(data.lock.locked_at ?? new Date().toISOString())}
						</span>
					{:else}
						<span style="color:var(--text-dim);">Not locked — there is nothing to check.</span>
					{/if}
				</div>
				<div style="font-size:12px;color:var(--text-dim);">
					verified on the site: {formatDurationShort(data.total_verified_secs)}
				</div>
			</div>

			{#if stale && !data.today}
				<div style="margin-top:12px;font-size:13px;color:var(--gold);">
					No check since {data.metrics.last_check_at ? timeAgo(data.metrics.last_check_at) : 'the lock started'} —
					the hours stopped being evidence. Are you still in it?
				</div>
			{/if}

			{#if data.today}
				<div style="margin-top:14px;padding:12px;border:1px solid var(--border);border-radius:6px;">
					<div style="font-size:12px;letter-spacing:0.12em;color:var(--text-dim);">TODAY'S SEAL</div>
					<div style="font-family:var(--font-mono);font-size:26px;letter-spacing:0.22em;color:var(--green);margin:4px 0;">
						{data.today.seal_code}
					</div>
					<div style="font-size:12px;color:var(--text-dim);">
						Status: <b style="color:var(--text);">{data.today.status}</b>
						· {data.today.confirms} confirm{data.today.confirms === 1 ? '' : 's'}
						· {data.today.disputes} dispute{data.today.disputes === 1 ? '' : 's'}
						· claims {formatDuration(data.today.claimed_secs)}
					</div>
				</div>
			{:else if data.lock.locked}
				<div style="margin-top:14px;display:flex;gap:10px;flex-wrap:wrap;">
					<input
						placeholder="What is it doing to you? (optional, 140 chars)"
						maxlength="140"
						bind:value={note}
						style="flex:1;min-width:200px;"
					/>
					<button class="btn btn-red" onclick={check} disabled={busy}>
						{busy ? 'CHECKING…' : 'CHECK THE CAGE'}
					</button>
				</div>
			{/if}
		</div>

		<div class="stat-grid" style="margin-bottom:16px;">
			<div class="stat-card">
				<div class="label">Verified hours</div>
				<div class="value green">{formatDurationShort(data.metrics.verified_secs)}</div>
			</div>
			<div class="stat-card">
				<div class="label">Claimed</div>
				<div class="value">{formatDurationShort(data.metrics.claimed_secs)}</div>
			</div>
			<div class="stat-card">
				<div class="label">Verification rate</div>
				<div class="value gold">{Math.round(data.metrics.verification_rate * 100)}%</div>
			</div>
			<div class="stat-card">
				<div class="label">Verified day streak</div>
				<div class="value">{data.metrics.current_verified_day_streak}<span style="font-size:14px;color:var(--text-dim);">d</span></div>
			</div>
			<div class="stat-card">
				<div class="label">Checks</div>
				<div class="value">{data.metrics.checks}</div>
			</div>
			<div class="stat-card">
				<div class="label">Longest verified run</div>
				<div class="value">{data.metrics.longest_verified_day_streak}d</div>
			</div>
		</div>

		<h2 style="font-size:16px;letter-spacing:0.08em;margin:24px 0 12px;">
			WAITING FOR A VOICE ({queue.length})
		</h2>
		<div class="card" style="padding:4px 18px;">
			{#if queue.length === 0}
				<div class="empty">
					<div class="big">🔒</div>
					Nothing to verify right now.
					<div style="font-size:12px;margin-top:10px;">
						When someone checks in, their seal appears here for you to compare.
					</div>
				</div>
			{:else}
				{#each queue as c (c.id)}
					<div style="padding:12px 0;border-bottom:1px solid var(--border);">
						<div style="display:flex;justify-content:space-between;gap:12px;flex-wrap:wrap;">
							<div>
								<a href={`/profile/${c.username}`} style="font-weight:600;">{c.username}</a>
								<span style="color:var(--text-dim);font-size:12px;">
									· checked {timeAgo(c.checked_at)} · claims {formatDuration(c.claimed_secs)}
								</span>
							</div>
							<div style="font-family:var(--font-mono);font-size:14px;letter-spacing:0.18em;color:var(--green);">
								{c.seal_code}
							</div>
						</div>
						{#if c.note}
							<div style="font-size:13px;color:var(--text-dim);font-style:italic;margin-top:6px;">
								“{c.note}”
							</div>
						{/if}
						<div style="display:flex;gap:8px;margin-top:10px;">
							<button
								class="btn btn-red"
								style="padding:4px 12px;font-size:12px;"
								onclick={() => attest(c.id, 'confirm')}
								disabled={attesting === c.id}
							>
								CONFIRM
							</button>
							<button
								class="btn btn-ghost"
								style="padding:4px 12px;font-size:12px;"
								onclick={() => attest(c.id, 'dispute')}
								disabled={attesting === c.id}
							>
								DISPUTE
							</button>
							{#if c.my_verdict}
								<span style="font-size:12px;color:var(--text-dim);align-self:center;">
									you said: {c.my_verdict}
								</span>
							{/if}
						</div>
					</div>
				{/each}
			{/if}
		</div>

		{#if data.recent.length > 0}
			<h2 style="font-size:16px;letter-spacing:0.08em;margin:24px 0 12px;">MY RECENT CHECKS</h2>
			<div class="card" style="padding:4px 18px;">
				{#each data.recent as c (c.id)}
					<div style="display:flex;justify-content:space-between;gap:12px;padding:10px 0;border-bottom:1px solid var(--border);font-size:13px;">
						<span style="color:var(--text-dim);">{timeAgo(c.checked_at)}</span>
						<span style="font-family:var(--font-mono);letter-spacing:0.18em;">{c.seal_code}</span>
						<span class:gold={c.status === 'pending'} class:green={c.status === 'verified'} class:red={c.status === 'disputed'}>
							{c.status}
						</span>
						<span>{formatDuration(c.claimed_secs)}</span>
					</div>
				{/each}
			</div>
		{/if}
	{/if}
</div>
