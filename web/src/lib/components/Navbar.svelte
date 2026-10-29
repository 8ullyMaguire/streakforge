<script lang="ts">
	import { page } from '$app/state';
	import { api, ApiRequestError } from '$lib/api';
	import type { SessionUser } from '$lib/types';
	import { pushToast } from '$lib/toasts.svelte';
	import { Flame, Heart, Lock } from 'lucide-svelte';

	let user = $state<SessionUser | null>(null);
	let locked = $state(false);
	let loaded = $state(false);

	$effect(() => {
		// fetch once on mount
		if (loaded) return;
		loaded = true;
		api
			.me()
			.then((u) => {
				user = u;
			})
			.catch((e) => {
				if (e instanceof ApiRequestError && e.status === 401) {
					user = null;
				} else {
					user = null;
				}
			});
	});

	// poll lock state when signed in (for the navbar lock badge)
	$effect(() => {
		if (!user) return;
		let stopped = false;
		api
			.lockState()
			.then((l) => {
				if (!stopped) locked = l.locked;
			})
			.catch(() => {});
		const t = setInterval(() => {
			if (stopped) return;
			api
				.lockState()
				.then((l) => {
					if (!stopped) locked = l.locked;
				})
				.catch(() => {});
		}, 30000);
		return () => {
			stopped = true;
			clearInterval(t);
		};
	});

	async function logout() {
		try {
			await api.logout();
			user = null;
			pushToast('Logged out.');
			// hard nav to landing to clear any cached state
			window.location.href = '/';
		} catch (e) {
			pushToast(e instanceof Error ? e.message : 'Logout failed', 'error');
		}
	}

	const links = [
		{ href: '/dashboard', label: 'MY PLACE' },
		{ href: '/denial', label: 'DENIAL' },
		// The cage check and the metrics are the two surfaces the owner asked for on
		// 2026-10-29; a feature nobody can find is a feature that does not exist.
		{ href: '/cage', label: 'CAGE' },
		// The drill's lines are written by members now, so the deck has its own door.
		{ href: '/drill', label: 'DRILL' },
		{ href: '/affirmations', label: 'DECK' },
		{ href: '/leaderboard', label: 'BOARD' },
		{ href: '/metrics', label: 'NUMBERS' },
		{ href: '/feed', label: 'FEED' },
		{ href: '/doctrine', label: 'DOCTRINE' }
	];
</script>

<nav class="nav">
	<div class="container nav-inner">
		<a href="/" class="nav-brand"><Flame size={20} class="flame" /> STREAKFORGE</a>
		<div class="nav-links">
			{#each links as l}
				<a href={l.href} class="nav-link" class:active={page.url.pathname.startsWith(l.href)}>
					{l.label}
				</a>
			{/each}
		</div>
		<div class="nav-spacer"></div>
		<!-- Support link. Deliberately OUTSIDE .nav-links: that block is
		     display:none below 720px, and a support ask that vanishes on phones is
		     the one place it should not be. Icon-only, so it costs no width next to
		     the sign-in button; the title/aria-label carry the meaning instead. -->
		<a
			class="nav-heart"
			href="https://liberapay.com/Critical_Nexus/"
			target="_blank"
			rel="noopener"
			title="Support StreakForge on Liberapay"
			aria-label="Support StreakForge on Liberapay"
		>
			<Heart size={18} fill="currentColor" />
		</a>
		{#if user}
			{#if locked}
				<a href="/denial" class="nav-lock-badge" title="You are locked">🔒</a>
			{/if}
			<a href="/settings" class="nav-link">{user.username}</a>
			{#if user.is_guest}
				<!--
					GS-1: a guest holds a real identity with no credential, so the way out
					of it is registration rather than a password change. Saying so in the
					navbar is what stops "where is my account" being a question.
				-->
				<a
					href="/login"
					class="nav-link"
					style="color:var(--gold);letter-spacing:0.08em;font-size:11px;"
					title="Guest identity — register to keep this history">GUEST</a
				>
			{/if}
			<button class="btn btn-ghost" onclick={logout}>Log out</button>
		{:else}
			<a href="/login" class="btn btn-red">SIGN IN</a>
		{/if}
	</div>
</nav>

<!--
	No <slot /> here. The only use is `<Navbar />` in +layout.svelte, which
	passes no children, so the slot rendered nothing and only cost a Svelte 5
	deprecation warning. If Navbar ever needs to project page content, add
	`let { children } = $props()` and `{@render children?.()}` instead.
-->

<style>
	.nav-heart {
		display: inline-flex;
		align-items: center;
		color: var(--red);
		opacity: 0.7;
		transition: opacity 0.15s, transform 0.06s;
	}
	.nav-heart:hover {
		opacity: 1;
		transform: scale(1.1);
	}
	.nav-lock-badge {
		margin-right: 8px;
		font-size: 15px;
		text-decoration: none;
		opacity: 0.9;
	}
	.nav-lock-badge:hover {
		opacity: 1;
	}
</style>
