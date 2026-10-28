<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiRequestError } from '$lib/api';
	import { computeChallengeProof } from '$lib/challenge';
	import { isValidUsername } from '$lib/validation';
	import { pushToast } from '$lib/toasts.svelte';
	import { page } from '$app/state';
	import { safeNext } from '$lib/deeplink';

	// Where to go after a successful sign-in. Read once at mount: the query
	// string cannot change under us mid-login, and re-reading it during submit
	// would make the redirect depend on render timing.
	const next = safeNext(page.url.searchParams.get('next'));

	type Mode = 'login' | 'register';

	let mode = $state<Mode>('login');
	let username = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let website = $state(''); // honeypot — hidden, must stay empty
	let error = $state<string | null>(null);
	let busy = $state(false);
	// GS-1: the door is offered only when the server says it is open. Asking first
	// means a closed door is never a button that fails.
	let guestOpen = $state(false);
	let guestLimit = $state(0);
	const formOpenedAt = Date.now();

	onMount(async () => {
		try {
			const status = await api.guestStatus();
			guestOpen = status.enabled && !status.at_cap;
			guestLimit = status.limit;
		} catch {
			guestOpen = false;
		}
	});

	async function continueAsGuest() {
		if (busy) return;
		busy = true;
		error = null;
		try {
			await api.mintGuest();
			pushToast('You are in as a guest. Register any time to keep this history.');
			window.location.href = next;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Could not open a guest session.';
			busy = false;
		}
	}

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
			window.location.href = next;
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

		{#if guestOpen}
			<!--
				GS-1: a guest is a real identity with no credential, and its history is
				folded into the account on registration — so this is not a throwaway
				session, and the sentence has to say so or nobody spends an evening on it.
			-->
			<div style="margin-top:20px;padding-top:16px;border-top:1px solid rgba(255,255,255,0.08);">
				<button
					class="btn btn-ghost btn-block"
					onclick={continueAsGuest}
					disabled={busy}
					type="button"
				>
					{busy ? 'OPENING…' : 'LOOK AROUND WITHOUT AN ACCOUNT'}
				</button>
				<p style="color:var(--text-dim);font-size:12px;margin:8px 0 0;line-height:1.5;">
					A guest identity, no password. What you log as a guest moves into your account
					when you register — the streak is not lost, because it was never a demo. Guests
					are capped at {guestLimit} active accounts site-wide.
				</p>
			</div>
		{/if}
	</div>
</div>
