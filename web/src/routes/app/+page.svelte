<script lang="ts">
	import { Download } from 'lucide-svelte';
</script>

<svelte:head>
	<title>Install — StreakForge</title>
</svelte:head>

<div class="container" style="max-width:720px;padding-top:28px;">
	<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 4px;">INSTALL THE APP</h1>
	<p style="color:var(--text-dim);font-size:14px;margin:0 0 20px;">
		No download needed. StreakForge runs in your browser — add it to your home
		screen and it opens like a real app, with its own icon and no address bar.
	</p>

	<div class="card" style="padding:18px 20px;margin-bottom:16px;">
		<h2 style="font-size:16px;letter-spacing:0.06em;margin:0 0 8px;">🦊 Firefox (Android)</h2>
		<ol style="margin:0;padding-left:20px;color:var(--text);font-size:14px;line-height:1.8;">
			<li>Open <a href="/" style="color:var(--red);">streakforge.polarisocial.xyz</a></li>
			<li>Tap the <strong>three dots</strong> menu (top right)</li>
			<li>Tap <strong>More</strong></li>
			<li>Tap <strong>Add app to home screen</strong></li>
			<li>Confirm — the app icon appears on your home screen</li>
		</ol>
	</div>

	<div class="card" style="padding:18px 20px;margin-bottom:16px;">
		<h2 style="font-size:16px;letter-spacing:0.06em;margin:0 0 8px;">🌐 Chrome / Android</h2>
		<ol style="margin:0;padding-left:20px;color:var(--text);font-size:14px;line-height:1.8;">
			<li>Open the site</li>
			<li>Tap the <strong>three dots</strong> menu (top right)</li>
			<li>Tap <strong>Add to Home screen</strong> (or <strong>Install app</strong>)</li>
			<li>Tap <strong>Install</strong> / <strong>Add</strong></li>
		</ol>
	</div>

	<div class="card" style="padding:18px 20px;margin-bottom:16px;">
		<h2 style="font-size:16px;letter-spacing:0.06em;margin:0 0 8px;">🍎 iPhone / iPad (Safari)</h2>
		<ol style="margin:0;padding-left:20px;color:var(--text);font-size:14px;line-height:1.8;">
			<li>Open the site in Safari</li>
			<li>Tap the <strong>Share</strong> button (box with arrow, bottom)</li>
			<li>Tap <strong>Add to Home Screen</strong></li>
			<li>Tap <strong>Add</strong> (top right)</li>
		</ol>
	</div>

	<div class="card" style="padding:18px 20px;margin-bottom:16px;">
		<h2 style="font-size:16px;letter-spacing:0.06em;margin:0 0 8px;">💻 Desktop (any browser)</h2>
		<ol style="margin:0;padding-left:20px;color:var(--text);font-size:14px;line-height:1.8;">
			<li>Open the site</li>
			<li>Look for the <strong>install icon</strong> in the address bar (Chrome/Edge: monitor-with-plus icon)</li>
			<li>Click it, then <strong>Install</strong></li>
			<li>Or: Chrome menu → <strong>Cast, save and share</strong> → <strong>Install page as app</strong></li>
		</ol>
	</div>

	<p style="color:var(--text-dim);font-size:13px;text-align:center;margin-top:20px;">
		<Download size={14} style="display:inline;vertical-align:-2px;" />
		The app works offline-ish, remembers you (30 days), and keeps your lock
		streak ticking. Deny harder.
	</p>
</div>
