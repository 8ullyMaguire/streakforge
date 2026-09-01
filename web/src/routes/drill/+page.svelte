<script lang="ts">
	import { api, ApiRequestError, formatCount } from '$lib/api';
	import type { Stats } from '$lib/types';
	import { pushToast } from '$lib/toasts.svelte';
	import { AFFIRMATIONS } from '$lib/affirmations';
	import { matchesAffirmation, affirmationSimilarity } from '$lib/typo';
	import { onMount } from 'svelte';
	import StreakCalendar from '$lib/components/StreakCalendar.svelte';
	import { Repeat, ChevronRight } from 'lucide-svelte';

	let stats = $state<Stats | null>(null);
	let error = $state<string | null>(null);
	let logging = $state(false);
	let index = $state(0);
	let typed = $state('');
	let matched = $derived(matchesAffirmation(typed, AFFIRMATIONS[index].text));
	let similarity = $derived(affirmationSimilarity(typed, AFFIRMATIONS[index].text));

	onMount(async () => {
		try {
			const res = await api.drill();
			stats = res.stats;
		} catch (e) {
			if (e instanceof ApiRequestError && e.status === 401) {
				window.location.href = '/login';
				return;
			}
			error = e instanceof Error ? e.message : 'Failed to load drill';
		}
	});

	function next() {
		typed = '';
		index = (index + 1) % AFFIRMATIONS.length;
	}
	function prev() {
		typed = '';
		index = (index - 1 + AFFIRMATIONS.length) % AFFIRMATIONS.length;
	}

	async function repeat() {
		if (logging || !matched) return;
		logging = true;
		try {
			const res = await api.logHabit(undefined, 'affirmation');
			stats = res.stats;
			typed = '';
			pushToast('Affirmation drilled. Repeat.');
		} catch (e) {
			if (e instanceof ApiRequestError && e.status === 429) {
				pushToast(e.message, 'error');
			} else {
				pushToast(e instanceof Error ? e.message : 'Failed to log affirmation', 'error');
			}
		} finally {
			logging = false;
		}
	}

	function digitClass(i: number, len: number) {
		if (i < 3) return 'digit-red';
		if (i >= len - 3) return 'digit-green';
		return 'digit-black';
	}

	function timeUntil(iso: string | null): string {
		if (!iso) return 'soon';
		const diff = new Date(iso).getTime() - Date.now();
		if (diff <= 0) return 'now';
		const mins = Math.ceil(diff / 60000);
		if (mins < 60) return `in ~${mins} min`;
		const h = Math.floor(mins / 60);
		return `in ~${h}h ${mins % 60}m`;
	}

	const calendarDays = $derived(
		stats
			? (() => {
					const byDay = new Map<string, number>();
					for (const l of stats.recent_logs) {
						const d = new Date(l.logged_at);
						const key = d.toISOString().slice(0, 10);
						byDay.set(key, (byDay.get(key) ?? 0) + 1);
					}
					const out: { date: string; count: number }[] = [];
					const now = new Date();
					for (let i = 29; i >= 0; i--) {
						const d = new Date(
							Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate() - i)
						);
						const key = d.toISOString().slice(0, 10);
						out.push({ date: key, count: byDay.get(key) ?? 0 });
					}
					return out;
				})()
			: []
	);
</script>

<svelte:head>
	<title>Affirmation Drill — StreakForge</title>
</svelte:head>

<div class="container" style="max-width:720px;padding-top:28px;">
	{#if error && !stats}
		<div class="card empty" style="color:var(--red);">{error}</div>
	{:else if !stats}
		<div class="skeleton" style="height:400px;"></div>
	{:else}
		<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 4px;">AFFIRMATION DRILL</h1>
		<p style="color:var(--text-dim);font-size:14px;margin:0 0 20px;">
			Type the mantra. Speak it with your own hands. One rep per hour. Five per day. Own your submission.
		</p>

		<!-- The counter (wlw-style) -->
		<div style="text-align:center;padding:24px 0 8px;">
			<div class="counter counter-digits" aria-label="Total affirmations {stats.alltime_count}" style="font-size:64px;">
				{#each formatCount(stats.alltime_count).split('') as ch, i (i)}
					<span class={digitClass(i, 9)}>{ch}</span>
				{/each}
			</div>
			<p style="color:var(--text-dim);font-size:12px;letter-spacing:0.2em;text-transform:uppercase;margin:8px 0 0;">
				affirmations drilled
			</p>
		</div>

		<!-- The affirmation card -->
		<div class="card" style="margin:20px 0;text-align:center;position:relative;overflow:hidden;">
			<div class="crown" style="font-size:28px;">🖤</div>
			<div style="font-size:20px;font-weight:700;line-height:1.4;min-height:80px;display:flex;align-items:center;justify-content:center;padding:12px 8px;">
				{AFFIRMATIONS[index].text}
			</div>
			<p style="color:var(--text-dim);font-size:12px;letter-spacing:0.12em;text-transform:uppercase;margin:0;">
				{AFFIRMATIONS[index].source}
			</p>

			<!-- Typed drill: must type the affirmation before REPEAT enables -->
			<div style="margin:18px 0 6px;text-align:left;">
				<label for="drill-input" style="display:block;font-size:12px;letter-spacing:0.1em;text-transform:uppercase;color:var(--text-dim);margin-bottom:8px;">
					TYPE THE AFFIRMATION
				</label>
				<input
					id="drill-input"
					type="text"
					bind:value={typed}
					placeholder="type it out…"
					autocomplete="off"
					autocapitalize="off"
					spellcheck="false"
					style="width:100%;padding:12px 14px;font-size:15px;border:1px solid var(--border);border-radius:10px;background:var(--bg-card);color:var(--text);"
					class:input-match={matched}
				/>
				{#if typed.length > 0 && !matched}
					<p style="color:var(--text-dim);font-size:12px;margin:8px 0 0;">
						Match: {Math.round(similarity * 100)}% — keep typing, whiteboi
					</p>
				{:else if matched}
					<p style="color:var(--green);font-size:12px;margin:8px 0 0;font-weight:600;">
						✓ MATCHED. Now repeat it.
					</p>
				{/if}
			</div>

			<div style="display:flex;justify-content:space-between;align-items:center;margin-top:14px;">
				<button class="btn btn-ghost" onclick={prev} aria-label="Previous affirmation">
					<ChevronRight size={16} style="transform:rotate(180deg);" />
				</button>
				<button
					class="btn btn-red btn-lg"
					onclick={repeat}
					disabled={logging || !stats.can_log || !matched}
					style="flex:1;margin:0 12px;"
				>
					<Repeat size={18} /> {logging ? 'DRILLING…' : matched ? 'REPEAT' : 'TYPE IT FIRST'}
				</button>
				<button class="btn btn-ghost" onclick={next} aria-label="Next affirmation">
					<ChevronRight size={16} />
				</button>
			</div>
			{#if !stats.can_log && stats.next_allowed_at}
				<p style="color:var(--text-dim);font-size:12px;margin-top:10px;">
					Next rep allowed {timeUntil(stats.next_allowed_at)}
				</p>
			{/if}
		</div>

		<!-- Stats -->
		<div class="stat-grid" style="margin:20px 0;">
			<div class="stat-card">
				<div class="label">Today</div>
				<div class="value green">{stats.today_count}</div>
			</div>
			<div class="stat-card">
				<div class="label">Drill streak</div>
				<div class="value red">{stats.current_streak}d</div>
			</div>
			<div class="stat-card">
				<div class="label">Longest</div>
				<div class="value gold">{stats.longest_streak}d</div>
			</div>
			<div class="stat-card">
				<div class="label">This week</div>
				<div class="value">{stats.week_count}</div>
			</div>
		</div>

		<h2 style="font-size:16px;letter-spacing:0.08em;margin:0 0 12px;">LAST 30 DAYS</h2>
		<div class="card" style="margin-bottom:24px;">
			<StreakCalendar days={calendarDays} />
		</div>
	{/if}
</div>

<style>
	.input-match {
		border-color: rgba(34, 197, 94, 0.6) !important;
		box-shadow: 0 0 0 2px rgba(34, 197, 94, 0.12);
	}
</style>
