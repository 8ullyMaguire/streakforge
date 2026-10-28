<script lang="ts">
	import { api } from '$lib/api';
	import type { ManifestoDoc } from '$lib/types';
	import { renderMarkdown } from '$lib/markdown';
	import { Heart } from 'lucide-svelte';
	import { onMount } from 'svelte';

	let docs = $state<ManifestoDoc[]>([]);
	let activeId = $state<string | null>(null);
	let content = $state<string | null>(null);
	let loading = $state(false);
	let error = $state<string | null>(null);

	onMount(async () => {
		try {
			const res = await api.manifestoList();
			docs = res.docs;
			if (docs.length > 0) {
				await select(docs[0].id);
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load doctrine';
		}
	});

	async function select(id: string) {
		activeId = id;
		loading = true;
		error = null;
		try {
			content = await api.manifestoDoc(id);
		} catch (e) {
			content = null;
			error = e instanceof Error ? e.message : 'Failed to load document';
		} finally {
			loading = false;
		}
	}

	let rendered = $derived(content ? renderMarkdown(content) : '');
</script>

<svelte:head>
	<title>Doctrine — StreakForge</title>
</svelte:head>

<div class="container" style="padding-top:28px;">
	<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 4px;">THE DOCTRINE</h1>
	<p style="color:var(--text-dim);font-size:14px;margin:0 0 20px;">
		Doctrine. Commandments. Training. The collected texts. For the vocabulary the
		texts assume — WLW, WLD, a denial, a drill rep, the devotion index — the
		<a href="/glossary">glossary</a> defines each one and names the rule that
		enforces it.
	</p>

	{#if error && docs.length === 0}
		<div class="card empty" style="color:var(--red);">{error}</div>
	{:else}
		<div style="display:flex;gap:20px;flex-wrap:wrap;align-items:flex-start;">
			<!-- Sidebar -->
			<aside style="flex:0 0 240px;min-width:200px;position:sticky;top:76px;max-height:calc(100vh - 100px);overflow-y:auto;">
				<div class="card" style="padding:8px;">
					{#each docs as d}
						<button
							class="doc-link"
							class:active={activeId === d.id}
							onclick={() => select(d.id)}
							style="display:block;width:100%;text-align:left;padding:10px 12px;border:none;background:transparent;color:var(--text-dim);font-size:14px;border-radius:8px;cursor:pointer;transition:background .15s,color .15s;"
						>
							{d.title}
						</button>
					{/each}
				</div>
			</aside>

			<!-- Content -->
			<div style="flex:1;min-width:280px;">
				{#if loading}
					<div class="skeleton" style="height:400px;"></div>
				{:else if error}
					<div class="card empty" style="color:var(--red);">{error}</div>
				{:else if content !== null}
					<article class="md-body card" style="line-height:1.65;font-size:15px;">
						{@html rendered}
					</article>
				{:else}
					<div class="card empty">
						<div class="big">📜</div>
						Select a text from the list.
					</div>
				{/if}

				<!-- Support ask, at the foot of the text. Placed here rather than in a
				     modal or a one-shot banner: the doctrine is what people came for, and
				     finishing a text is the moment the ask reads as an offer rather than a
				     toll. Rendered in every state below the list, so it survives a doc
				     that fails to load. -->
				<section class="card support">
					<h2>KEEP THE FORGE LIT</h2>
					<p>
						StreakForge is one person's project — no ads, no sponsors, no upsell. The
						server, the backups and the texts are paid for out of pocket, and the drills
						stay free for everyone. If the doctrine is doing its work on you, put
						something back.
					</p>
					<a
						class="btn btn-red"
						href="https://liberapay.com/Critical_Nexus/"
						target="_blank"
						rel="noopener"
					>
						<Heart size={16} fill="currentColor" /> SUPPORT ON LIBERAPAY
					</a>
				</section>
			</div>
		</div>
	{/if}
</div>

<style>
	.support {
		margin-top: 20px;
		border-left: 3px solid var(--red);
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 12px;
	}
	.support h2 {
		font-size: 16px;
		letter-spacing: 0.06em;
		margin: 0;
		color: var(--gold);
	}
	.support p {
		margin: 0;
		color: var(--text-dim);
		font-size: 14px;
		line-height: 1.6;
	}
	.doc-link:hover {
		background: rgba(255, 255, 255, 0.06);
		color: var(--text);
	}
	.doc-link.active {
		background: rgba(227, 28, 35, 0.15);
		color: var(--text);
		border-left: 3px solid var(--red);
	}
	.doc-link.active:hover {
		background: rgba(227, 28, 35, 0.2);
	}
	:global(.md-body h1) {
		font-size: 24px;
		margin: 0 0 12px;
		letter-spacing: 0.02em;
	}
	:global(.md-body h2) {
		font-size: 20px;
		margin: 28px 0 10px;
		color: var(--gold);
		letter-spacing: 0.02em;
	}
	:global(.md-body h3) {
		font-size: 17px;
		margin: 22px 0 8px;
		color: var(--red-soft);
	}
	:global(.md-body p) {
		margin: 10px 0;
	}
	:global(.md-body a) {
		color: var(--red);
		text-decoration: underline;
		text-underline-offset: 2px;
	}
	:global(.md-body a:hover) {
		color: var(--red);
	}
	:global(.md-body ul) {
		padding-left: 22px;
		margin: 10px 0;
	}
	:global(.md-body li) {
		margin: 6px 0;
	}
	:global(.md-body blockquote) {
		border-left: 3px solid var(--red);
		margin: 12px 0;
		padding: 8px 14px;
		background: rgba(227, 28, 35, 0.06);
		color: var(--text-dim);
		border-radius: 0 8px 8px 0;
	}
	:global(.md-body strong) {
		color: var(--text);
	}
	:global(.md-body code) {
		font-family: var(--font-mono);
		font-size: 13px;
		background: var(--bg-elev);
		padding: 1px 5px;
		border-radius: 4px;
	}
	:global(.md-body pre) {
		background: var(--bg-elev);
		border: 1px solid var(--border);
		border-radius: 10px;
		padding: 14px;
		overflow-x: auto;
	}
	:global(.md-body pre code) {
		background: none;
		padding: 0;
	}
	:global(.md-body hr) {
		border: none;
		border-top: 1px solid var(--border);
		margin: 20px 0;
	}
	:global(.md-body em) {
		color: var(--text-dim);
	}
</style>
