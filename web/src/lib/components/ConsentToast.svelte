<script lang="ts">
	import { onMount } from 'svelte';
	import {
		OPTIONAL_CATEGORIES,
		consentRequired,
		fetchConsentState,
		giveConsent,
		refuseConsent,
		refusedThisTab,
		shouldAsk,
		type ConsentState
	} from '$lib/consent';

	// GS-3. A banner, not a modal: the site is fully readable behind it, and the
	// gate it fronts is the API's. Blocking the page in order to ask permission
	// to read it would be the dark pattern the standard warns about.
	let visible = $state(false);
	let busy = $state(false);
	let error = $state('');
	let current: ConsentState | null = $state(null);
	let chosen: Record<string, boolean> = $state(
		Object.fromEntries(OPTIONAL_CATEGORIES.map((c) => [c, false]))
	);

	async function ask(): Promise<void> {
		current = await fetchConsentState();
		visible = shouldAsk(current, refusedThisTab());
	}

	onMount(() => {
		void ask();
		// A 403 from anywhere in the app reopens the question: the visitor may
		// have clicked a button whose answer arrived after their cookie expired.
		const unsubscribe = consentRequired.subscribe((required) => {
			if (required) visible = true;
		});
		return unsubscribe;
	});

	async function accept(all: boolean): Promise<void> {
		busy = true;
		error = '';
		try {
			const categories = all
				? [...OPTIONAL_CATEGORIES]
				: OPTIONAL_CATEGORIES.filter((c) => chosen[c]);
			current = await giveConsent(categories);
			visible = false;
		} catch (e) {
			// The refusal that matters is server-side; if the request failed the
			// honest thing is to say so rather than pretend consent was recorded.
			error = e instanceof Error ? e.message : 'consent could not be recorded';
		} finally {
			busy = false;
		}
	}

	async function decline(): Promise<void> {
		busy = true;
		error = '';
		try {
			await refuseConsent();
			visible = false;
		} catch {
			// A refusal that failed to reach the server changed nothing, so hide
			// the banner anyway: nothing was stored and nothing was claimed.
			visible = false;
		} finally {
			busy = false;
		}
	}
</script>

{#if visible}
	<!--
		The wording is deliberately plain and the three answers are all one click.
		"Essential only" is not a lesser answer that needs hunting for: a site that
		only offers "accept everything" has not asked a question.
	-->
	<div
		role="dialog"
		aria-live="polite"
		aria-labelledby="consent-title"
		class="card"
		style="position:fixed;left:12px;right:12px;bottom:12px;z-index:55;max-width:620px;margin:0 auto;padding:16px;border-color:rgba(212,175,55,0.35);background:var(--bg, var(--bg));"
	>
		<div
			id="consent-title"
			style="font-weight:700;letter-spacing:0.08em;font-size:13px;margin-bottom:8px;"
		>
			COOKIES
		</div>
		<p style="margin:0 0 10px;font-size:13px;color:var(--text-dim);line-height:1.5;">
			This site keeps a session cookie so you stay signed in — that one is not optional.
			Analytics and personalisation are off unless you turn them on. Declining leaves
			everything readable; you just cannot post or register until you accept.
			<a href="/terms" style="color:inherit;text-decoration:underline;">What is stored</a>
		</p>

		<div style="display:flex;flex-wrap:wrap;gap:8px;align-items:center;">
			{#each OPTIONAL_CATEGORIES as category (category)}
				<label style="display:flex;gap:6px;align-items:center;font-size:12px;letter-spacing:0.04em;color:var(--text-dim);">
					<input
						type="checkbox"
						bind:checked={chosen[category]}
						disabled={busy}
						style="accent-color:var(--gold, var(--gold));"
					/>
					{category.toUpperCase()}
				</label>
			{/each}
		</div>

		<div style="display:flex;flex-wrap:wrap;gap:8px;margin-top:12px;">
			<button class="btn" onclick={() => accept(false)} disabled={busy}>
				SAVE CHOICES
			</button>
			<button class="btn btn-ghost" onclick={() => accept(true)} disabled={busy}>
				ACCEPT ALL
			</button>
			<button
				class="btn btn-ghost"
				onclick={decline}
				disabled={busy}
				style="margin-left:auto;"
			>
				DECLINE
			</button>
		</div>

		{#if error}
			<div style="margin-top:8px;font-size:12px;color:var(--danger);">{error}</div>
		{/if}
	</div>
{/if}
