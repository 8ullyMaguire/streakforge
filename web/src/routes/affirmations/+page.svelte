<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiRequestError, timeAgo } from '$lib/api';
	import { pushToast } from '$lib/toasts.svelte';
	import type { DeckItem, DeckResponse } from '$lib/types';

	// THE DECK — the lines the drill reads out, and where they come from.
	//
	// Until 2026-10-29 this list lived in a TypeScript array, which meant the only person
	// who could add a line was the person who could deploy. Now anyone can write one; the
	// trust ladder decides whether it goes straight into the deck or waits for a level-2
	// voice, and the page says which of those happened instead of leaving people guessing
	// why their line is not where they expected it.

	let data = $state<DeckResponse | null>(null);
	let error = $state<string | null>(null);
	let body = $state('');
	let busy = $state(false);
	let working = $state<number | null>(null);

	const remaining = $derived(data?.remaining_today ?? null);
	const canSubmit = $derived(
		remaining !== null && remaining > 0 && body.trim().length >= 3 && !busy
	);

	async function load() {
		try {
			data = await api.deck();
			error = null;
		} catch (e) {
			error =
				e instanceof ApiRequestError || e instanceof Error ? e.message : 'Could not load the deck.';
		}
	}

	onMount(load);

	async function submit() {
		if (!canSubmit) return;
		busy = true;
		try {
			const res = await api.submitAffirmation(body.trim());
			body = '';
			pushToast(
				res.live
					? 'In the deck. It will come up in the drill.'
					: 'Queued — a level 2 member admits it to the deck.'
			);
			await load();
		} catch (e) {
			pushToast(e instanceof Error ? e.message : 'Could not submit that line.', 'error');
		} finally {
			busy = false;
		}
	}

	async function act(id: number, kind: 'approve' | 'hide' | 'delete') {
		working = id;
		try {
			if (kind === 'approve') await api.approveAffirmation(id);
			else if (kind === 'hide') await api.hideAffirmation(id);
			else await api.deleteAffirmation(id);
			pushToast(
				kind === 'approve'
					? 'Admitted to the deck.'
					: kind === 'hide'
						? 'Hidden. It is out of the drill.'
						: 'Deleted.'
			);
			await load();
		} catch (e) {
			pushToast(e instanceof Error ? e.message : 'That did not work.', 'error');
		} finally {
			working = null;
		}
	}

	function statusLabel(item: DeckItem) {
		if (item.status === 'pending') return 'WAITING';
		if (item.status === 'hidden') return 'HIDDEN';
		return item.source === 'member' ? 'MEMBER' : item.source.toUpperCase();
	}
</script>

<svelte:head>
	<title>The deck — StreakForge</title>
</svelte:head>

<div class="container" style="max-width:720px;padding-top:28px;">
	<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 8px;">THE DECK</h1>
	<p style="color:var(--text-dim);font-size:13px;margin:0 0 18px;line-height:1.6;">
		These are the lines the drill reads out and asks you to type back. The first ones came
		from the guides this site was built out of; the rest are written by members. Level 2
		members write straight into the deck — below that, a line waits for one to admit it.
	</p>

	{#if error}
		<div class="card" style="padding:12px 14px;margin-bottom:16px;color:var(--red-soft);font-size:13px;">
			{error}
		</div>
	{/if}

	{#if data}
		<div class="card" style="padding:16px;margin-bottom:20px;">
			<div style="font-size:12px;letter-spacing:0.12em;color:var(--text-dim);margin-bottom:10px;">
				ADD A LINE
			</div>
			{#if data.remaining_today === null}
				<div style="font-size:13px;color:var(--text-dim);">
					<a href="/login" style="color:var(--gold);">Sign in</a> to add your own — a guest
					session can write one too, and it follows you when you register.
				</div>
			{:else if remaining === 0}
				<div style="font-size:13px;color:var(--text-dim);">
					Today's three are written. Come back tomorrow.
				</div>
			{:else}
				<textarea
					placeholder="One line, the way you would say it out loud."
					maxlength={data.max_len}
					rows="2"
					bind:value={body}
					style="width:100%;resize:vertical;font-family:var(--font-body);"
				></textarea>
				<div style="display:flex;justify-content:space-between;align-items:center;gap:12px;margin-top:8px;flex-wrap:wrap;">
					<div style="font-size:11px;color:var(--text-dim);">
						{body.trim().length}/{data.max_len} · {remaining} left today ·
						{#if data.level >= data.review_level}
							it goes straight into the deck
						{:else}
							a level {data.review_level} member admits it
						{/if}
					</div>
					<button class="btn btn-red" onclick={submit} disabled={!canSubmit}>
						{busy ? 'WRITING…' : 'ADD TO THE DECK'}
					</button>
				</div>
			{/if}
		</div>

		{#if data.review.length > 0}
			<h2 style="font-size:15px;letter-spacing:0.08em;margin:0 0 10px;">
				WAITING FOR A VOICE ({data.review.length})
			</h2>
			<div class="card" style="padding:6px 14px;margin-bottom:20px;">
				{#each data.review as item (item.id)}
					<div style="display:flex;gap:12px;align-items:center;padding:10px 0;border-bottom:1px solid var(--border);">
						<div style="flex:1;">
							<div style="font-size:14px;">{item.body}</div>
							<div style="font-size:11px;color:var(--text-dim);margin-top:3px;">
								{item.author} · {timeAgo(item.created_at)}
							</div>
						</div>
						<button
							class="btn"
							onclick={() => act(item.id, 'approve')}
							disabled={working === item.id}
						>
							ADMIT
						</button>
						<button
							class="btn"
							style="color:var(--text-dim);"
							onclick={() => act(item.id, 'hide')}
							disabled={working === item.id}
						>
							NO
						</button>
					</div>
				{/each}
			</div>
		{/if}

		{#if data.mine.length > 0}
			<h2 style="font-size:15px;letter-spacing:0.08em;margin:0 0 10px;">YOUR LINES</h2>
			<div class="card" style="padding:6px 14px;margin-bottom:20px;">
				{#each data.mine as item (item.id)}
					<div style="display:flex;gap:12px;align-items:center;padding:10px 0;border-bottom:1px solid var(--border);">
						<div style="flex:1;">
							<div style="font-size:14px;color:{item.status === 'hidden' ? 'var(--text-dim)' : 'var(--text)'};">
								{item.body}
							</div>
							<div style="font-size:11px;color:var(--text-dim);margin-top:3px;">
								{statusLabel(item)} · {timeAgo(item.created_at)}
							</div>
						</div>
						{#if item.status !== 'hidden'}
							<button class="btn" onclick={() => act(item.id, 'hide')} disabled={working === item.id}>
								HIDE
							</button>
						{/if}
						<button
							class="btn"
							style="color:var(--red-soft);"
							onclick={() => act(item.id, 'delete')}
							disabled={working === item.id}
						>
							DELETE
						</button>
					</div>
				{/each}
			</div>
		{/if}

		<h2 style="font-size:15px;letter-spacing:0.08em;margin:0 0 10px;">
			IN THE DECK ({data.deck.length})
		</h2>
		<div class="card" style="padding:6px 14px;">
			{#each data.deck as item (item.id)}
				<div style="padding:10px 0;border-bottom:1px solid var(--border);">
					<div style="font-size:14px;">{item.body}</div>
					<div style="font-size:11px;color:var(--text-dim);margin-top:3px;letter-spacing:0.06em;">
						{statusLabel(item)}{item.mine ? '' : ` · ${item.author}`}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>
