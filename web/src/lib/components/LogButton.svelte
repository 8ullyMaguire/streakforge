<script lang="ts">
	import { api, ApiRequestError } from '$lib/api';
	import type { Stats, LogKind } from '$lib/types';
	import { pushToast } from '$lib/toasts.svelte';
	import { remainingSecs, formatCountdown } from '$lib/countdown';
	import { Zap } from 'lucide-svelte';

	interface Props {
		stats: () => Stats;
		onLogged?: (s: Stats) => void;
		/** Label + kind for the submit button. Defaults to the waste action. */
		action?: { label: string; kind?: LogKind; notePlaceholder?: string };
	}
	let { stats, onLogged, action }: Props = $props();

	// $derived, not const: `action` is a reactive prop, and reading it once at
	// init would freeze the kind if a caller ever swapped the action.
	let kind = $derived<LogKind>(action?.kind ?? 'habit');
	let buttonLabel = $derived(action?.label ?? 'WASTE A LOAD');

	let logging = $state(false);
	let note = $state('');

	// Live countdown. `receivedAt` is when the current stats payload arrived, so
	// the remaining seconds tick down from the SERVER's number rather than
	// trusting the browser clock to agree with it. A bare setInterval that only
	// bumped a counter would still read correctly; the baseline is what makes a
	// reload or a backgrounded tab resync instead of drifting.
	let receivedAt = $state(Date.now());
	let tick = $state(0);

	$effect(() => {
		// Re-baseline whenever a new payload lands (stats() is reactive here).
		stats();
		receivedAt = Date.now();
	});

	$effect(() => {
		const t = setInterval(() => {
			tick++;
		}, 1000);
		return () => clearInterval(t);
	});

	let serverSecs = $derived(stats()?.next_allowed_in_secs ?? 0);
	let blocked = $derived(stats()?.blocked_by_exclusivity ?? false);
	let remaining = $derived(remainingSecs(serverSecs, Date.now() - receivedAt));
	let canLog = $derived((stats()?.can_log ?? false) && remaining <= 0 && !blocked);

	async function log() {
		if (logging || !canLog) return;
		logging = true;
		try {
			const res = await api.logHabit(note.trim() || undefined, kind);
			note = '';
			onLogged?.(res.stats);
			pushToast('Recorded. The board remembers.');
		} catch (e) {
			if (e instanceof ApiRequestError && e.status === 429) {
				pushToast(e.message, 'error');
			} else {
				pushToast(e instanceof Error ? e.message : 'Failed to log', 'error');
			}
		} finally {
			logging = false;
		}
	}
</script>

<div style="display:flex;flex-direction:column;gap:12px;">
	{#if blocked}
		<p style="color:var(--red);font-size:13px;margin:0;">
			{#if kind === 'denial'}
				You already recorded a waste today — WLWs and WLDs are mutually exclusive, so the
				denial stands blocked until midnight UTC.
			{:else}
				You already recorded a denial today — WLWs and WLDs are mutually exclusive, so the
				waste stands blocked until midnight UTC.
			{/if}
		</p>
	{:else if !canLog && remaining > 0}
		<p style="color:var(--text-dim);font-size:13px;margin:0;">
			One submission per day (UTC). Your streak is safe — it resets at midnight.
		</p>
	{/if}

	<textarea
		placeholder={action?.notePlaceholder ?? 'Confession (max 140 chars)…'}
		maxlength="140"
		bind:value={note}
		style="width:100%;min-height:70px;resize:vertical;"
	></textarea>

	<button class="log-btn" onclick={log} disabled={logging || !canLog} aria-label={buttonLabel}>
		<Zap size={22} style="vertical-align:-3px;" /> {buttonLabel}
	</button>

	{#if !canLog && remaining > 0}
		<div style="text-align:center;margin-top:2px;">
			<div
				style="font-family:var(--font-mono);font-size:22px;letter-spacing:0.08em;color:var(--text);"
				aria-label="Time until your next submission is allowed"
			>
				{formatCountdown(remaining)}
			</div>
			<p style="color:var(--text-dim);font-size:12px;margin:2px 0 0;">
				Next submission allowed at 00:00 UTC
			</p>
		</div>
	{/if}
</div>
