<script lang="ts">
	import '../app.css';
	import Navbar from '$lib/components/Navbar.svelte';
	import ToastStack from '$lib/components/ToastStack.svelte';
</script>

<Navbar />
<main>
	<slot />
</main>
<ToastStack />

<footer class="footer">
	<div class="container">STREAKFORGE — EMBRACE THE STREAK. THE FUTURE IS CONSISTENT.</div>
</footer>
