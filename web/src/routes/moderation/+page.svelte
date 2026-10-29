<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiRequestError } from '$lib/api';
	import { pushToast } from '$lib/toasts.svelte';
	import type { ModerationQueueItem, Trust } from '$lib/types';

	// ORDER — trust levels and quorum, made visible.
	//
	// The owner's rule: "all moderation through discourse-style trust levels or
	// quorum". This page is therefore two things and no more: where you stand, and
	// what is waiting for a verdict. It says what each level unlocks, because a
	// level nobody can explain is a level nobody trusts.

	const LEVELS: { level: number; name: string; unlocks: string; needs: string }[] = [
		{ level: 0, name: 'NEW', unlocks: 'read and log', needs: 'any account' },
		{ level: 1, name: 'MEMBER', unlocks: 'flag content, verify cage checks', needs: 'a day old, 3 logs' },
		{ level: 2, name: 'TRUSTED', unlocks: 'the queue, restore votes', needs: '7 active days, 10 contributions' },
		{ level: 3, name: 'REGULAR', unlocks: 'one voice verifies a cage check', needs: '30 active days, 40 contributions' },
		{ level: 4, name: 'LEADER', unlocks: 'hide and restore alone, audited', needs: 'granted by hand' }
	];

	let trust = $state<Trust | null>(null);
	let queue = $state<ModerationQueueItem[]>([]);
	let error = $state<string | null>(null);
	let busy = $state<number | null>(null);

	async function load() {
		try {
			trust = await api.trust();
			if (trust.level >= 2) {
				queue = await api.moderationQueue();
			}
			error = null;
		} catch (e) {
			error = e instanceof ApiRequestError || e instanceof Error ? e.message : 'Could not load.';
		}
	}

	onMount(load);

	async function act(item: ModerationQueueItem, action: 'hide' | 'restore') {
		busy = item.target_id;
		try {
			const res = await api.moderationAction(item.target_kind, item.target_id, action);
			pushToast(
				action === 'hide'
					? 'Hidden. The action is on the record.'
					: res.hidden
						? `Vote recorded — ${res.restore_votes} of 2.`
						: 'Restored.'
			);
			await load();
		} catch (e) {
			pushToast(e instanceof Error ? e.message : 'Could not do that.', 'error');
		} finally {
			busy = null;
		}
	}
</script>

<svelte:head>
	<title>Order — StreakForge</title>
</svelte:head>

<div class="container" style="max-width:720px;padding-top:28px;">
	<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 8px;">ORDER</h1>
	<p style="color:var(--text-dim);font-size:13px;margin:0 0 20px;line-height:1.6;">
		Nobody here can hide something quietly. Three flags — at least one from a trusted member —
		hide content; two restore votes bring it back; a leader can act alone, and every action is
		recorded with who did it.
	</p>

	{#if error}
		<div class="card" style="padding:12px 14px;margin-bottom:16px;color:var(--red-soft);font-size:13px;">
			{error}
		</div>
	{/if}

	{#if trust}
		<div class="card" style="padding:16px;margin-bottom:16px;">
			<div style="display:flex;justify-content:space-between;gap:12px;flex-wrap:wrap;align-items:baseline;">
				<div>
					<div style="font-size:12px;letter-spacing:0.12em;color:var(--text-dim);">YOUR TRUST LEVEL</div>
					<div style="font-size:26px;color:var(--gold);">
						TL{trust.level} · {LEVELS[trust.level]?.name ?? '—'}
					</div>
				</div>
				<div style="font-size:12px;color:var(--text-dim);text-align:right;">
					<div>{trust.active_days} active days · {trust.contributions} contributions</div>
					{#if trust.next_level_at !== null}
						<div style="margin-top:4px;color:var(--text);">
							{trust.next_level_at} more contribution{trust.next_level_at === 1 ? '' : 's'} to the next level
						</div>
					{:else}
						<div style="margin-top:4px;">Level 4 is granted by hand, not earned by volume.</div>
					{/if}
					{#if trust.upheld_flags > 0}
						<div style="margin-top:4px;color:var(--red-soft);">{trust.upheld_flags} upheld flag(s) against you in 30 days</div>
					{/if}
				</div>
			</div>

			<div style="margin-top:14px;border-top:1px solid var(--border);padding-top:12px;">
				{#each LEVELS as l (l.level)}
					<div
						style="display:flex;gap:10px;font-size:12px;padding:4px 0;color:{l.level === trust.level ? 'var(--text)' : 'var(--text-dim)'};"
					>
						<span style="width:42px;font-family:var(--font-mono);">TL{l.level}</span>
						<span style="width:76px;letter-spacing:0.06em;">{l.name}</span>
						<span style="flex:1;">{l.unlocks}</span>
						<span style="color:var(--text-dim);">{l.needs}</span>
					</div>
				{/each}
			</div>
		</div>

		{#if trust.level >= 2}
			<h2 style="font-size:16px;letter-spacing:0.08em;margin:24px 0 12px;">
				THE QUEUE ({queue.length})
			</h2>
			<div class="card" style="padding:4px 18px;">
				{#if queue.length === 0}
					<div class="empty">
						<div class="big">⚖️</div>
						Nothing flagged.
					</div>
				{:else}
					{#each queue as item (item.target_kind + item.target_id)}
						<div style="padding:12px 0;border-bottom:1px solid var(--border);">
							<div style="display:flex;justify-content:space-between;gap:12px;flex-wrap:wrap;font-size:13px;">
								<span>
									<b>{item.target_kind === 'log' ? 'LOG' : 'CAGE CHECK'}</b>
									by <a href={`/profile/${item.author ?? ''}`}>{item.author ?? 'unknown'}</a>
									{#if item.hidden}<span style="color:var(--red);">· HIDDEN</span>{/if}
								</span>
								<span style="color:var(--text-dim);">
									{item.flags} flags ({item.from_tl2} trusted)
								</span>
							</div>
							{#if item.preview}
								<div style="font-size:13px;color:var(--text-dim);font-style:italic;margin-top:6px;">
									“{item.preview}”
								</div>
							{/if}
							<div style="display:flex;gap:8px;margin-top:10px;">
								{#if item.hidden}
									<button class="btn btn-red" style="padding:4px 12px;font-size:12px;" onclick={() => act(item, 'restore')} disabled={busy === item.target_id}>
										RESTORE (2 VOTES)
									</button>
								{:else if trust.is_leader}
									<button class="btn btn-ghost" style="padding:4px 12px;font-size:12px;" onclick={() => act(item, 'hide')} disabled={busy === item.target_id}>
										HIDE
									</button>
								{:else}
									<span style="font-size:12px;color:var(--text-dim);">
										Three flags hide it; leaders can act now.
									</span>
								{/if}
							</div>
						</div>
					{/each}
				{/if}
			</div>
		{/if}
	{/if}
</div>
