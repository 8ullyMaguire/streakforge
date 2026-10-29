<script lang="ts">
	import { api, ApiRequestError } from '$lib/api';
	import type { SessionUser } from '$lib/types';
	import { pushToast } from '$lib/toasts.svelte';
	import { onMount } from 'svelte';

	let me = $state<SessionUser | null>(null);
	let username = $state('');
	let displayName = $state('');
	let avatarUrl = $state('');
	let socialUrl = $state('');
	// Confessions are public by default (owner, 2026-10-29). The switch is the only
	// thing standing between "the site publishes what I write" and "it does not", so
	// it is stated in full here rather than as a bare checkbox label.
	let logsPublic = $state(true);
	let saving = $state(false);
	let error = $state<string | null>(null);

	// The data rights, in the two directions the terms promise: take it, or have it
	// erased. Erasure is the only irreversible action on the site, so it needs the
	// password and a typed word, and it says exactly what happens first.
	let eraseOpen = $state(false);
	let erasePassword = $state('');
	let eraseConfirm = $state('');
	let erasing = $state(false);
	let eraseError = $state<string | null>(null);

	async function erase() {
		if (erasing) return;
		erasing = true;
		eraseError = null;
		try {
			await api.eraseAccount(erasePassword, eraseConfirm);
			// The account is gone and the session with it: leave the member on a page
			// that can render without one.
			window.location.href = '/?erased=1';
		} catch (e) {
			eraseError =
				e instanceof ApiRequestError || e instanceof Error
					? e.message
					: 'Could not erase the account.';
			erasing = false;
		}
	}

	onMount(async () => {
		try {
			me = await api.me();
			username = me.username;
			displayName = me.display_name ?? '';
			avatarUrl = me.avatar_url ?? '';
			socialUrl = me.social_url ?? '';
			const p = await api.profile(me.username);
			logsPublic = p.logs_public;
		} catch (e) {
			if (e instanceof ApiRequestError && e.status === 401) {
				window.location.href = '/login';
				return;
			}
			error = e instanceof Error ? e.message : 'Not signed in';
		}
	});

	async function save() {
		saving = true;
		error = null;
		try {
			const p = await api.updateProfile({
				username: username.trim() || undefined,
				display_name: displayName.trim() || undefined,
				avatar_url: avatarUrl.trim() || undefined,
				social_url: socialUrl.trim() || undefined,
				logs_public: logsPublic
			});
			pushToast('Profile updated.');
			if (me) me.username = p.username;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to update';
			pushToast(error, 'error');
		} finally {
			saving = false;
		}
	}
</script>

<svelte:head>
	<title>Settings — StreakForge</title>
</svelte:head>

<div class="container" style="max-width:480px;padding-top:28px;">
	<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 20px;">SETTINGS</h1>
	{#if error && !me}
		<div class="card empty" style="color:var(--red);">{error}</div>
	{:else if !me}
		<div class="skeleton" style="height:200px;"></div>
	{:else}
		<div class="card">
			<div class="form-row">
				<label for="username">Username (public)</label>
				<input id="username" bind:value={username} maxlength="30" />
			</div>
			<div class="form-row">
				<label for="display">Display name</label>
				<input id="display" bind:value={displayName} maxlength="50" />
			</div>
			<div class="form-row">
				<label for="avatar">Avatar URL</label>
				<input id="avatar" bind:value={avatarUrl} placeholder="https://…" />
				<div style="margin-top:8px;">
					<img
						class="avatar avatar-md"
						src={avatarUrl || `https://api.dicebear.com/7.x/identicon/svg?seed=${username || 'anon'}`}
						alt=""
					/>
				</div>
			</div>
			<div class="form-row">
				<label for="logs-public">Confessions</label>
				<label style="display:flex;gap:10px;align-items:flex-start;font-size:13px;line-height:1.5;">
					<input
						id="logs-public"
						type="checkbox"
						bind:checked={logsPublic}
						style="margin-top:3px;width:auto;"
					/>
					<span>
						<b>Publish my confessions in the public feed.</b>
						<span style="color:var(--text-dim);">
							On by default. Turn it off and your logs stay yours — the rows and your
							score are untouched, they just stop appearing on the feed.
						</span>
					</span>
				</label>
			</div>
			<div class="form-row">
				<label for="social">Social URL</label>
				<input id="social" bind:value={socialUrl} placeholder="https://github.com/you" />
				<div style="margin-top:6px;color:var(--text-dim);font-size:12px;">
					Shown on your public profile. http(s) link or empty.
				</div>
			</div>
			{#if error}
				<div class="form-error">{error}</div>
			{/if}
			<button class="btn btn-red btn-block" onclick={save} disabled={saving}>
				{saving ? 'SAVING…' : 'SAVE'}
			</button>
		</div>

		<div class="card" style="margin-top:20px;padding:18px;">
			<div style="font-size:12px;letter-spacing:0.12em;color:var(--text-dim);margin-bottom:10px;">YOUR DATA</div>

			<div style="display:flex;justify-content:space-between;gap:14px;flex-wrap:wrap;align-items:center;">
				<div style="font-size:13px;color:var(--text-dim);max-width:360px;line-height:1.55;">
					Everything the site holds about you — account, logs, locks, cage checks, attestations,
					flags, audit events — as one JSON file.
				</div>
				<a class="btn btn-ghost" href={api.exportUrl} download>DOWNLOAD MY DATA</a>
			</div>

			<div style="border-top:1px solid var(--border);margin:16px 0 14px;"></div>

			{#if !eraseOpen}
				<div style="display:flex;justify-content:space-between;gap:14px;flex-wrap:wrap;align-items:center;">
					<div style="font-size:13px;color:var(--text-dim);max-width:360px;line-height:1.55;">
						Erasing deletes the account and every log, lock, check and vote with it. It cannot
						be undone and there is no export afterwards.
					</div>
					<button class="btn btn-ghost" style="border-color:rgba(200,30,30,0.5);color:var(--red-soft);" onclick={() => (eraseOpen = true)}>
						ERASE ACCOUNT
					</button>
				</div>
			{:else}
				<div style="font-size:13px;line-height:1.6;">
					<p style="color:var(--red-soft);margin:0 0 10px;">
						This deletes your account and everything in it: logs, locks, cage checks,
						attestations and flags. Anonymous audit rows saying an erasure happened remain.
					</p>
					<div class="form-row">
						<label for="erase-password">Your password</label>
						<input id="erase-password" type="password" bind:value={erasePassword} autocomplete="current-password" />
					</div>
					<div class="form-row">
						<label for="erase-confirm">Type DELETE to confirm</label>
						<input id="erase-confirm" bind:value={eraseConfirm} placeholder="DELETE" autocomplete="off" />
					</div>
					{#if eraseError}
						<div class="form-error">{eraseError}</div>
					{/if}
					<div style="display:flex;gap:8px;flex-wrap:wrap;">
						<button
							class="btn btn-red"
							onclick={erase}
							disabled={erasing || eraseConfirm !== 'DELETE' || !erasePassword}
						>
							{erasing ? 'ERASING…' : 'ERASE EVERYTHING'}
						</button>
						<button class="btn btn-ghost" onclick={() => { eraseOpen = false; erasePassword = ''; eraseConfirm = ''; eraseError = null; }}>
							CANCEL
						</button>
					</div>
				</div>
			{/if}
		</div>
	{/if}
</div>
