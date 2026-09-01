<script lang="ts">
	import { api, ApiRequestError, formatCount } from '$lib/api';
	import type { DenialResponse } from '$lib/types';
	import { pushToast } from '$lib/toasts.svelte';
	import { onMount } from 'svelte';
	import { Lock, Unlock, Ban, Crown } from 'lucide-svelte';

	let data = $state<DenialResponse | null>(null);
	let error = $state<string | null>(null);
	let acting = $state(false);
	let unlockReason = $state('self');
	let now = $state(Date.now());

	onMount(() => {
		load();
		const t = setInterval(() => {
			now = Date.now();
		}, 1000);
		return () => {
			clearInterval(t);
		};
	});

	async function load() {
		try {
			data = await api.denial();
		} catch (e) {
			if (e instanceof ApiRequestError && e.status === 401) {
				window.location.href = '/login';
				return;
			}
			error = e instanceof Error ? e.message : 'Failed to load denial page';
		}
	}

	async function lockSelf() {
		if (acting) return;
		acting = true;
		try {
			const lock = await api.lock();
			if (data) data.lock = lock;
			pushToast('Locked. Good whiteboi.');
		} catch (e) {
			pushToast(e instanceof Error ? e.message : 'Failed to lock', 'error');
		} finally {
			acting = false;
		}
	}

	async function unlockSelf() {
		if (acting) return;
		acting = true;
		try {
			const lock = await api.unlock(unlockReason);
			if (data) data.lock = lock;
			pushToast('Unlocked. Don\u2019t waste it.');
		} catch (e) {
			pushToast(e instanceof Error ? e.message : 'Failed to unlock', 'error');
		} finally {
			acting = false;
		}
	}

	async function reportDenial() {
		if (acting) return;
		acting = true;
		try {
			const res = await api.logHabit(undefined, 'denial');
			if (data) data.stats = res.stats;
			pushToast('Denial logged. Load denied. 🖤');
		} catch (e) {
			if (e instanceof ApiRequestError && e.status === 429) {
				pushToast(e.message, 'error');
			} else {
				pushToast(e instanceof Error ? e.message : 'Failed to log denial', 'error');
			}
		} finally {
			acting = false;
		}
	}

	function fmtDuration(secs: number): string {
		const s = Math.max(0, Math.floor(secs));
		const d = Math.floor(s / 86400);
		const h = Math.floor((s % 86400) / 3600);
		const m = Math.floor((s % 3600) / 60);
		const ss = s % 60;
		const pad = (n: number) => String(n).padStart(2, '0');
		if (d > 0) return `${d}d ${pad(h)}h ${pad(m)}m ${pad(ss)}s`;
		if (h > 0) return `${pad(h)}h ${pad(m)}m ${pad(ss)}s`;
		return `${pad(m)}m ${pad(ss)}s`;
	}

	function fmtLockStreak(n: number): string {
		return n === 1 ? '1 lock period' : `${n} lock periods`;
	}

	function digitClass(i: number, len: number) {
		if (i < 3) return 'digit-red';
		if (i >= len - 3) return 'digit-green';
		return 'digit-black';
	}

	// live ticking duration of the current lock
	let currentSecs = $derived(
		data && data.lock.locked && data.lock.locked_at
			? Math.floor((now - new Date(data.lock.locked_at).getTime()) / 1000)
			: 0
	);
</script>

<svelte:head>
	<title>Denial — StreakForge</title>
</svelte:head>

<div class="container" style="max-width:720px;padding-top:28px;">
	{#if error && !data}
		<div class="card empty" style="color:var(--red);">{error}</div>
	{:else if !data}
		<div class="skeleton" style="height:400px;"></div>
	{:else}
		<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 4px;">DENIAL</h1>
		<p style="color:var(--text-dim);font-size:14px;margin:0 0 20px;">
			A whiteboi who cums from his own hand wastes his seed. A whiteboi who is denied —
			who leaks from a plap, who ruins while pegged — denies it. Track both.
		</p>

		<!-- Lock status -->
		<div class="card" style="text-align:center;padding:24px;margin-bottom:20px;" class:locked={data.lock.locked}>
			{#if data.lock.locked}
				<div style="font-size:40px;margin-bottom:6px;">🔒</div>
				<p style="font-size:13px;letter-spacing:0.16em;text-transform:uppercase;color:var(--red);margin:0 0 6px;">
					LOCKED — {fmtLockStreak(data.lock.current_lock_streak)}
				</p>
				<div class="counter counter-digits" aria-label="Locked for {fmtDuration(currentSecs)}" style="font-size:44px;">
					{fmtDuration(currentSecs)}
				</div>
				<p style="color:var(--text-dim);font-size:12px;margin:8px 0 16px;">
					since {new Date(data.lock.locked_at ?? '').toLocaleString()}
				</p>
				<button class="btn btn-ghost" onclick={unlockSelf} disabled={acting}>
					<Unlock size={16} /> UNLOCK
				</button>
				<div style="margin-top:10px;font-size:12px;color:var(--text-dim);">
					<label for="unlock-reason" style="margin-right:6px;">reason</label>
					<select id="unlock-reason" bind:value={unlockReason} style="background:var(--bg-card);color:var(--text);border:1px solid var(--border);border-radius:6px;padding:4px 8px;">
						<option value="self">self</option>
						<option value="pegging">pegging</option>
						<option value="plapped">plapped</option>
						<option value="ruined">ruined</option>
						<option value="edge-marathon">edge-marathon</option>
						<option value="cleaned">cleaned</option>
					</select>
				</div>
			{:else}
				<div style="font-size:40px;margin-bottom:6px;">🔓</div>
				<p style="font-size:13px;letter-spacing:0.16em;text-transform:uppercase;color:var(--text-dim);margin:0 0 6px;">
					UNLOCKED
				</p>
				<p style="color:var(--text-dim);font-size:13px;margin:0 0 16px;">
					Your longest lock: {fmtDuration(data.lock.longest_lock_secs)}. Your total time locked: {fmtDuration(data.lock.total_locked_secs)}.
				</p>
				<button class="btn btn-red btn-lg" onclick={lockSelf} disabled={acting}>
					<Lock size={18} /> LOCK MYSELF
				</button>
			{/if}
		</div>

		<!-- Report a denial -->
		<div class="card" style="text-align:center;padding:24px;margin-bottom:20px;">
			<div style="font-size:32px;margin-bottom:6px;">💧</div>
			<h2 style="font-size:18px;letter-spacing:0.04em;margin:0 0 6px;">REPORT A DENIAL</h2>
			<p style="color:var(--text-dim);font-size:13px;margin:0 0 16px;">
				Edged and held. Leaked from a plap. Ruined while pegged. Say it.
			</p>
			<button
				class="btn btn-red btn-lg"
				onclick={reportDenial}
				disabled={acting || !!data.next_denial_allowed_in}
			>
				<Ban size={18} /> {acting ? 'LOGGING…' : 'I DENIED'}
			</button>
			{#if data.next_denial_allowed_in}
				<p style="color:var(--text-dim);font-size:12px;margin-top:10px;">
					Next denial reportable in {fmtDuration(data.next_denial_allowed_in)}
				</p>
			{/if}
		</div>

		<!-- Denial stats -->
		<div class="stat-grid" style="margin:20px 0;">
			<div class="stat-card">
				<div class="label">Today</div>
				<div class="value green">{data.stats.today_count}</div>
			</div>
			<div class="stat-card">
				<div class="label">Denial streak</div>
				<div class="value red">{data.stats.current_streak}d</div>
			</div>
			<div class="stat-card">
				<div class="label">Longest</div>
				<div class="value gold">{data.stats.longest_streak}d</div>
			</div>
			<div class="stat-card">
				<div class="label">All-time denied</div>
				<div class="value">{data.stats.alltime_count}</div>
			</div>
		</div>

		<!-- Lock stats -->
		<div class="stat-grid" style="margin:20px 0;">
			<div class="stat-card">
				<div class="label">Current lock streak</div>
				<div class="value red">{data.lock.current_lock_streak}</div>
			</div>
			<div class="stat-card">
				<div class="label">Longest lock streak</div>
				<div class="value gold">{data.lock.longest_lock_streak}</div>
			</div>
			<div class="stat-card">
				<div class="label">Total locked</div>
				<div class="value" style="font-size:14px;">{fmtDuration(data.lock.total_locked_secs)}</div>
			</div>
			<div class="stat-card">
				<div class="label">Longest single lock</div>
				<div class="value" style="font-size:14px;">{fmtDuration(data.lock.longest_lock_secs)}</div>
			</div>
		</div>

		<p style="color:var(--text-dim);font-size:12px;text-align:center;margin:8px 0 0;">
			<Crown size={12} style="display:inline;vertical-align:-2px;" /> The board rewards denial, not waste. Every denial counts 10x a wasted load.
		</p>
	{/if}
</div>

<style>
	.card.locked {
		border-color: rgba(227, 28, 35, 0.5);
		box-shadow: 0 0 24px rgba(227, 28, 35, 0.12);
	}
</style>
