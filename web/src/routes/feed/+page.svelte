<script lang="ts">
	import { api } from '$lib/api';
	import type { PublicFeedItem } from '$lib/types';
	import { timeAgo } from '$lib/api';
	import { onMount } from 'svelte';

	let items = $state<PublicFeedItem[]>([]);
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
	<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 12px;">
		PUBLIC ACTIVITY FEED
	</h1>
	<!--
		Item 01 used to say the opposite, and the reversal is the owner's call on
		2026-10-29: "confessions/logs are public by default, user can choose to make
		them private through a setting". So the page states the default out loud and
		points at the switch — the one thing that would make this dishonest is
		publishing without saying so.
	-->
	<p style="color:var(--text-dim);font-size:12px;margin:0 0 20px;line-height:1.6;">
		Confessions are public by default. <a href="/settings">Turn yours off</a> and your
		logs stay out of this feed — the rows, the streak and the score are untouched.
	</p>

	<div class="card" style="padding:4px 18px;">
		{#if items.length === 0 && !loading}
			<div class="empty">
				<div class="big">🕳️</div>
				No completions yet — be the first.
				<div style="font-size:12px;margin-top:10px;">
					The <a href="/glossary">glossary</a> explains the vocabulary, and every
					term there names the rule that enforces it.
				</div>
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
						<span style="color:{item.kind === 'denial' ? 'var(--gold)' : 'var(--text-dim)'};font-size:12px;">
							{item.kind === 'denial' ? ' denied a load' : ' wasted a load'}
						</span>
						{#if item.note}
							<div style="font-size:13px;color:var(--text);margin-top:4px;line-height:1.5;font-style:italic;">
								“{item.note}”
							</div>
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
