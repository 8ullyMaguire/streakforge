<script lang="ts">
	import { page } from '$app/state';
	import { api, ApiRequestError } from '$lib/api';
	import type { SessionUser } from '$lib/types';
	import { pushToast } from '$lib/toasts.svelte';
	import { Flame, Lock } from 'lucide-svelte';

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
		{ href: '/drill', label: 'DRILL' },
		{ href: '/leaderboard', label: 'BOARD' },
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
		{#if user}
			{#if locked}
				<a href="/denial" class="nav-lock-badge" title="You are locked">🔒</a>
			{/if}
			<a href="/settings" class="nav-link">{user.username}</a>
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
