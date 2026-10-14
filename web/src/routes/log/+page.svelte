<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { api, ApiRequestError } from '$lib/api';
	import { parseKind, kindLabel, clampNote, NOTE_MAX } from '$lib/deeplink';
	import { pushToast } from '$lib/toasts.svelte';
	import type { Stats } from '$lib/types';

	// This page exists so a log can be one tap from a phone shortcut or a
	// bookmark (docs/100_IDEAS.md #14). Everything it needs is in the URL, so
	// nothing here waits on a fetch except the sign-in check.
	const parsed = parseKind(page.url.searchParams.get('kind'));
	const note = clampNote(page.url.searchParams.get('note'));

	// The SIGNED-OUT state of a deep link is the common one — a shortcut gets
	// opened weeks after the session expired — so it is handled as a first-class
	// state, with the return path preserved through /login?next=.
	const returnTo = `/log?kind=${parsed.kind}${note ? `&note=${encodeURIComponent(note)}` : ''}`;

	let signedIn = $state<boolean | null>(null);
	let busy = $state(false);
	let result = $state<Stats | null>(null);
	let error = $state<string | null>(null);

	onMount(async () => {
		try {
			signedIn = !!(await api.me());
		} catch {
			signedIn = false;
		}
	});

	async function submit() {
		if (busy) return;
		busy = true;
		error = null;
		try {
			const res = await api.logHabit(note || undefined, parsed.kind);
			result = res.stats;
			pushToast('Recorded. The board remembers.');
		} catch (e) {
			// The daily budget and the exclusivity rule both answer 429 here,
			// and the server's message names the clock — show it as written.
			error =
				e instanceof ApiRequestError || e instanceof Error
					? e.message
					: 'Could not record that.';
		} finally {
			busy = false;
		}
	}
</script>

<svelte:head>
	<title>Log — StreakForge</title>
</svelte:head>

<div class="container" style="max-width:520px;padding-top:28px;">
	<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 16px;">RECORD IT</h1>

	{#if parsed.fellBack}
		<!-- Visible, not silent: recording a waste because a link typo'd the kind
		     would put something in a user's history they never asked for. -->
		<div
			class="card"
			style="margin-bottom:12px;padding:12px 14px;font-size:13px;line-height:1.5;color:var(--gold);border-color:rgba(212,175,55,0.35);"
		>
			That link's kind was not recognised, so this page is set to a waste.
		</div>
	{/if}

	<div class="card" style="padding:16px;">
		<div style="font-size:14px;margin:0 0 12px;line-height:1.55;">
			This will record <span style="color:var(--gold);">{kindLabel(parsed.kind)}</span>.
			{#if note}
				<div
					style="margin-top:8px;color:var(--text-dim);font-size:13px;font-style:italic;"
				>
					“{note}”
				</div>
			{/if}
		</div>

		{#if error}
			<div style="color:var(--red-soft);font-size:13px;margin-bottom:12px;line-height:1.5;">
				{error}
			</div>
		{/if}

		{#if result}
			<!-- The confirmation shows the server's numbers, not a claim about them. -->
			<div style="font-size:14px;line-height:1.6;">
				Recorded.
				<div style="color:var(--text-dim);font-size:13px;margin-top:6px;">
					{result.remaining_today} of {result.daily_limit} left today · lifetime
					<span style="color:var(--text);">{result.alltime_count}</span> logs
				</div>
			</div>
			<div style="margin-top:14px;display:flex;gap:10px;flex-wrap:wrap;">
				<a class="btn btn-ghost" href="/dashboard">DASHBOARD</a>
				<a class="btn btn-ghost" href={returnTo}>AGAIN</a>
			</div>
		{:else if signedIn === null}
			<div class="skeleton" style="height:44px;"></div>
		{:else if !signedIn}
			<a class="btn" href={`/login?next=${encodeURIComponent(returnTo)}`}>SIGN IN TO RECORD</a>
		{:else}
			<button class="btn" onclick={submit} disabled={busy}>
				{busy ? 'RECORDING…' : `LOG ${kindLabel(parsed.kind).toUpperCase()}`}
			</button>
		{/if}
	</div>

	{#if note.length >= NOTE_MAX}
		<p style="color:var(--text-dim);font-size:12px;margin-top:10px;">
			The note was shortened to {NOTE_MAX} characters, which is the limit the site accepts.
		</p>
	{/if}
</div>
