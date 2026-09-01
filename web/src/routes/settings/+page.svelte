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
	let saving = $state(false);
	let error = $state<string | null>(null);

	onMount(async () => {
		try {
			me = await api.me();
			username = me.username;
			displayName = me.display_name ?? '';
			avatarUrl = me.avatar_url ?? '';
			socialUrl = me.social_url ?? '';
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
				social_url: socialUrl.trim() || undefined
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
	{/if}
</div>
