<script lang="ts">
	import { onMount } from 'svelte';
	import { AGE_KEY, acceptedValue, cookieString, shouldGate } from '$lib/agegate';

	let visible = $state(false);

	onMount(() => {
		// localStorage is the primary store; the cookie covers a partitioned
		// storage context. Either one being set is enough to skip the ask.
		const stored = localStorage.getItem(AGE_KEY);
		const cookie = document.cookie.includes(`${AGE_KEY}=${acceptedValue()}`);
		visible = shouldGate(stored) && !cookie;
	});

	function accept() {
		try {
			localStorage.setItem(AGE_KEY, acceptedValue());
		} catch {
			// Private-mode storage can throw. The cookie below is the fallback,
			// and failing to persist entirely is not a reason to block the site.
		}
		document.cookie = cookieString();
		visible = false;
	}
</script>

{#if visible}
	<!--
		Item 08: this site is for adults and said so to nobody. The overlay is an
		acknowledgement, not a wall — it is dismissible and nothing server-side
		reads it — and /terms says exactly that rather than implying enforcement
		the site cannot make.
	-->
	<div
		role="dialog"
		aria-modal="true"
		aria-labelledby="agegate-title"
		style="position:fixed;inset:0;z-index:60;display:flex;align-items:center;justify-content:center;padding:20px;background:rgba(0,0,0,0.88);backdrop-filter:blur(3px);"
	>
		<div
			class="card"
			style="max-width:460px;padding:22px;border-color:rgba(212,175,55,0.35);"
		>
			<div
				id="agegate-title"
				style="font-size:13px;letter-spacing:0.12em;color:var(--gold);margin-bottom:10px;"
			>
				ADULTS ONLY
			</div>
			<p style="font-size:14px;line-height:1.6;margin:0 0 12px;">
				StreakForge is a personal tracking tool for adults. It records consensual
				things about the person using it and is not intended for minors.
			</p>
			<p style="font-size:12px;color:var(--text-dim);line-height:1.55;margin:0 0 16px;">
				By continuing you confirm you are 18 or older. This is a courtesy notice,
				not an age check — see the <a href="/terms">terms</a> for what the site does
				and does not do.
			</p>
			<button class="btn" onclick={accept}>I AM 18 OR OLDER</button>
		</div>
	</div>
{/if}
