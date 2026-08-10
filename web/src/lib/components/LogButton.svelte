<script lang="ts">
	import { api, ApiRequestError } from '$lib/api';
	import type { Stats } from '$lib/types';
	import { pushToast } from '$lib/toasts.svelte';
	import { Zap } from 'lucide-svelte';

	interface Props {
		stats: () => Stats;
		onLogged?: (s: Stats) => void;
	}
	let { stats, onLogged }: Props = $props();

	let logging = $state(false);
	let note = $state('');

	function timeUntil(iso: string | null): string {
		if (!iso) return 'soon';
		const diff = new Date(iso).getTime() - Date.now();
		if (diff <= 0) return 'now';
		const mins = Math.ceil(diff / 60000);
		if (mins < 60) return `in ~${mins} min`;
		const h = Math.floor(mins / 60);
		return `in ~${h}h ${mins % 60}m`;
	}

	async function log() {
		if (logging) return;
		logging = true;
		try {
			const res = await api.logHabit(note.trim() || undefined);
			note = '';
			onLogged?.(res.stats);
			pushToast('Completion logged. Forge on.');
		} catch (e) {
			if (e instanceof ApiRequestError && e.status === 429) {
				pushToast(e.message, 'error');
			} else {
				pushToast(e instanceof Error ? e.message : 'Failed to log completion', 'error');
			}
		} finally {
			logging = false;
		}
	}
</script>

<div style="display:flex;flex-direction:column;gap:12px;">
	{#if stats().last_60m > 0}
		<p style="color:var(--text-dim);font-size:13px;margin:0;">
			Logged {stats().last_60m}× in the last hour (limit 1/hr).
		</p>
	{/if}
	{#if stats().today_count >= 5}
		<p style="color:var(--red);font-size:13px;margin:0;">
			Daily limit reached (5/day).
		</p>
	{/if}
	<textarea
		placeholder="Optional note (max 140 chars)…"
		maxlength="140"
		bind:value={note}
		style="width:100%;min-height:70px;resize:vertical;"
	></textarea>
	<button
		class="log-btn"
		onclick={log}
		disabled={logging || !stats().can_log}
		aria-label="Log completion"
	>
		<Zap size={22} style="vertical-align:-3px;" /> LOG COMPLETION
	</button>
	{#if !stats().can_log && stats().next_allowed_at}
		<p style="color:var(--text-dim);font-size:12px;text-align:center;margin:0;">
			Next log allowed {timeUntil(stats().next_allowed_at)}
		</p>
	{/if}
</div>
