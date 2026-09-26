<script lang="ts">
	import '../app.css';
	import Navbar from '$lib/components/Navbar.svelte';
	import BottomNav from '$lib/components/BottomNav.svelte';
	import ToastStack from '$lib/components/ToastStack.svelte';
</script>

<Navbar />
<BottomNav />
<main>
	<slot />
</main>
<ToastStack />

<footer class="footer">
	<div class="container">
		STREAKFORGE — EMBRACE DEFEAT. THE FUTURE IS BLACK.
		<span style="margin:0 8px;">·</span>
		<a href="/app" style="color:var(--text-dim);text-decoration:underline;text-underline-offset:3px;">Install the app</a>
	</div>
</footer>
