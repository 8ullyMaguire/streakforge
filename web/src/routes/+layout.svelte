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
		<!-- The plan said to match the "Install the app" anchor's styling, but
		     that anchor is a ghost button and this is a secondary link beside it.
		     Two buttons in a footer would compete; a dimmed text link reads as the
		     quiet alternative the plan wanted. -->
		<div style="color:var(--text-dim);font-size:12px;letter-spacing:0.06em;margin-bottom:12px;">
			<a href="/kpi" style="color:var(--text-dim);">COMMUNITY KPI</a>
		</div>
		<div style="color:var(--text-dim);font-size:12px;letter-spacing:0.06em;">
			STREAKFORGE — EMBRACE DEFEAT. THE FUTURE IS BLACK.
		</div>
	</div>
</footer>
