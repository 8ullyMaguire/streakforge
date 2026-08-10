<script lang="ts">
	import { page } from '$app/state';
	import { api, ApiRequestError } from '$lib/api';
	import type { SessionUser } from '$lib/types';
	import { pushToast } from '$lib/toasts.svelte';
	import { Flame } from 'lucide-svelte';

	let user = $state<SessionUser | null>(null);
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
		{ href: '/dashboard', label: 'DASHBOARD' },
		{ href: '/drill', label: 'DRILL' },
		{ href: '/leaderboard', label: 'LEADERBOARD' },
		{ href: '/feed', label: 'FEED' },
		{ href: '/manifesto', label: 'MANIFESTO' }
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
		<div class="nav-spacer" />
		{#if user}
			<a href="/settings" class="nav-link">{user.username}</a>
			<button class="btn btn-ghost" onclick={logout}>Log out</button>
		{:else}
			<a href="/login" class="btn btn-red">SIGN IN</a>
		{/if}
	</div>
</nav>

<slot />
