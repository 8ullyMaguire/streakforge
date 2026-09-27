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
		<!-- PWA install is a real conversion path (manifest + icons ship), so it
		     gets a visible button rather than only a small underlined link. -->
		<a href="/app" class="btn btn-ghost" style="margin-bottom:12px;">INSTALL THE APP</a>
		<div style="color:var(--text-dim);font-size:12px;letter-spacing:0.06em;">
			STREAKFORGE — EMBRACE DEFEAT. THE FUTURE IS BLACK.
		</div>
	</div>
</footer>
