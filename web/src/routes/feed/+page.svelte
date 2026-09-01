<script lang="ts">
	import { api } from '$lib/api';
	import type { FeedItem } from '$lib/types';
	import { timeAgo } from '$lib/api';
	import { onMount } from 'svelte';

	let items = $state<FeedItem[]>([]);
	let cursor = $state<string | null>(null);
	let loading = $state(false);
	let done = $state(false);

	onMount(() => {
		loadMore();
	});

	async function loadMore() {
		if (loading || done) return;
		loading = true;
		try {
			const res = await api.feed(cursor ?? undefined);
			items = [...items, ...res.items];
			cursor = res.next_cursor;
			if (!cursor) done = true;
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Feed — StreakForge</title>
</svelte:head>

<div class="container" style="max-width:640px;padding-top:28px;">
	<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 20px;">
		PUBLIC ACTIVITY FEED
	</h1>

	<div class="card" style="padding:4px 18px;">
		{#if items.length === 0 && !loading}
			<div class="empty">
				<div class="big">🕳️</div>
				No completions yet — be the first.
			</div>
		{:else}
			{#each items as item (item.id)}
				<div class="feed-item">
					<img
						class="avatar avatar-md"
						src={item.avatar_url ?? `https://api.dicebear.com/7.x/identicon/svg?seed=${item.username}`}
						alt=""
					/>
					<div style="flex:1;">
						<span class="username" style="font-weight:600;">
							<a href={`/profile/${item.username}`}>{item.username}</a>
						</span>
						<span style="color:var(--text-dim);font-size:12px;"> logged a completion</span>
						{#if item.note}
							<div class="note">“{item.note}”</div>
						{/if}
					</div>
					<span class="time">{timeAgo(item.logged_at)}</span>
				</div>
			{/each}
		{/if}
		{#if loading}
			<div class="skeleton" style="height:60px;margin:12px 0;"></div>
		{/if}
	</div>

	{#if !done && items.length > 0}
		<div style="text-align:center;margin:20px 0;">
			<button class="btn btn-ghost" onclick={loadMore} disabled={loading}>
				{loading ? 'LOADING…' : 'LOAD MORE'}
			</button>
		</div>
	{/if}
</div>
