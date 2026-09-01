<script lang="ts">
	import { api, ApiRequestError } from '$lib/api';
	import { computeChallengeProof } from '$lib/challenge';
	import { isValidUsername } from '$lib/validation';
	import { pushToast } from '$lib/toasts.svelte';

	type Mode = 'login' | 'register';

	let mode = $state<Mode>('login');
	let username = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let website = $state(''); // honeypot — hidden, must stay empty
	let error = $state<string | null>(null);
	let busy = $state(false);
	const formOpenedAt = Date.now();

	function switchMode(next: Mode) {
		mode = next;
		error = null;
	}

	function validate(): string | null {
		if (!isValidUsername(username.trim())) {
			return 'Username must be 2–30 characters: letters, numbers, underscore.';
		}
		if (password.length < 8) {
			return 'Password must be at least 8 characters.';
		}
		if (mode === 'register' && password !== confirmPassword) {
			return 'Passwords do not match.';
		}
		return null;
	}

	async function submit() {
		if (busy) return;
		error = null;
		const problem = validate();
		if (problem) {
			error = problem;
			return;
		}
		busy = true;
		try {
			const { nonce } = await api.nonce();
			const challengeProof = await computeChallengeProof(nonce, username.trim(), password);
			const payload = {
				username: username.trim(),
				password,
				form_opened_at: formOpenedAt,
				website,
				challenge_proof: challengeProof,
				challenge_nonce: nonce
			};
			await (mode === 'register' ? api.register(payload) : api.login(payload));
			pushToast(mode === 'register' ? 'Account created. Welcome!' : 'Signed in.');
			window.location.href = '/dashboard';
		} catch (e) {
			error = e instanceof ApiRequestError ? e.message : e instanceof Error ? e.message : 'Something went wrong.';
			busy = false;
		}
	}
</script>

<svelte:head>
	<title>Sign in — StreakForge</title>
</svelte:head>

<div class="container">
	<div class="card auth-card">
		<h1 style="font-size:26px;letter-spacing:0.04em;margin:0 0 8px;">SIGN IN</h1>
		<p style="color:var(--text-dim);font-size:14px;margin:0 0 24px;">
			Own your submission. No external accounts needed.
		</p>

		<div class="auth-tabs" role="tablist">
			<button
				class="auth-tab"
				class:active={mode === 'login'}
				role="tab"
				aria-selected={mode === 'login'}
				onclick={() => switchMode('login')}
			>
				SIGN IN
			</button>
			<button
				class="auth-tab"
				class:active={mode === 'register'}
				role="tab"
				aria-selected={mode === 'register'}
				onclick={() => switchMode('register')}
			>
				REGISTER
			</button>
		</div>

		<form onsubmit={(e) => { e.preventDefault(); submit(); }}>
			<div class="form-row">
				<label for="username">Username</label>
				<input
					id="username"
					name="username"
					bind:value={username}
					autocomplete="username"
					maxlength="30"
					spellcheck="false"
					required
				/>
			</div>
			<div class="form-row">
				<label for="password">Password</label>
				<input
					id="password"
					name="password"
					type="password"
					bind:value={password}
					autocomplete={mode === 'register' ? 'new-password' : 'current-password'}
					minlength="8"
					required
				/>
			</div>
			{#if mode === 'register'}
				<div class="form-row">
					<label for="confirm-password">Confirm password</label>
					<input
						id="confirm-password"
						name="confirm-password"
						type="password"
						bind:value={confirmPassword}
						autocomplete="new-password"
						minlength="8"
						required
					/>
				</div>
			{/if}

			<!-- Honeypot: hidden from humans, bots fill it in. Must be submitted empty. -->
			<div class="honeypot" aria-hidden="true">
				<label for="website">Website</label>
				<input
					id="website"
					name="website"
					type="text"
					tabindex="-1"
					autocomplete="off"
					bind:value={website}
				/>
			</div>

			{#if error}
				<div class="form-error">{error}</div>
			{/if}

			<button class="btn btn-red btn-block" type="submit" disabled={busy}>
				{busy ? 'WORKING…' : mode === 'register' ? 'CREATE ACCOUNT' : 'SIGN IN'}
			</button>
		</form>

		<p style="color:var(--text-dim);font-size:12px;margin-top:18px;">
			{mode === 'register'
				? 'Already have an account? Use the SIGN IN tab.'
				: 'New here? Switch to REGISTER to claim your username.'}
		</p>
	</div>
</div>
