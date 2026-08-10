<script lang="ts">
	import { api, ApiRequestError } from '$lib/api';
	import { pushToast } from '$lib/toasts.svelte';

	let error = $state<string | null>(null);
	let busy = $state(false);
	const isDev = import.meta.env.DEV || (import.meta.env.VITE_ALLOW_DEV_LOGIN ?? false);

	async function devLogin() {
		if (busy) return;
		busy = true;
		error = null;
		try {
			await api.me();
			pushToast('Already signed in.');
			window.location.href = '/dashboard';
			return;
		} catch {
			// not signed in — fall through to dev login
		}
		try {
			await fetch('/api/auth/dev-login', { method: 'POST', credentials: 'include' });
			pushToast('Signed in with dev account.');
			window.location.href = '/dashboard';
		} catch (e) {
			error = e instanceof Error ? e.message : 'Dev login failed';
			busy = false;
		}
	}
</script>

<svelte:head>
	<title>Sign in — StreakForge</title>
</svelte:head>

<div class="container">
	<div class="card auth-card" style="text-align:center;">
		<h1 style="font-size:26px;letter-spacing:0.04em;margin:0 0 8px;">SIGN IN</h1>
		<p style="color:var(--text-dim);font-size:14px;margin:0 0 24px;">
			Own your submission. X login required.
		</p>

		<a href="/api/auth/x" class="btn btn-gold btn-block" style="margin-bottom:12px;">
			𝕏 SIGN IN WITH X
		</a>

		{#if isDev}
			<button class="btn btn-ghost btn-block" onclick={devLogin} disabled={busy}>
				{busy ? 'SIGNING IN…' : 'DEV LOGIN (local testing)'}
			</button>
		{/if}

		{#if error}
			<div class="form-error">{error}</div>
		{/if}

		<p style="color:var(--text-dim);font-size:12px;margin-top:18px;">
			New here? You'll be asked to pick a username after signing in.
		</p>
	</div>
</div>
