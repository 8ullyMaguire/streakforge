<script lang="ts">
	import { page } from '$app/state';
	import { Home, Ban, PenLine, Trophy, ScrollText } from 'lucide-svelte';

	const links = [
		{ href: '/dashboard', label: 'PLACE', icon: Home },
		{ href: '/denial', label: 'DENY', icon: Ban },
		{ href: '/drill', label: 'DRILL', icon: PenLine },
		{ href: '/leaderboard', label: 'BOARD', icon: Trophy },
		{ href: '/doctrine', label: 'TEXT', icon: ScrollText }
	];
</script>

<nav class="nav-bottom" aria-label="Mobile navigation">
	{#each links as l}
		<a
			href={l.href}
			class:active={page.url.pathname === l.href || page.url.pathname.startsWith(l.href + '/')}
		>
			<span class="icon"><l.icon size={18} /></span>
			{l.label}
		</a>
	{/each}
</nav>
