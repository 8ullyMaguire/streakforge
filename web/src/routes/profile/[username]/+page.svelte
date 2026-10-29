<script lang="ts">
	import { page } from '$app/state';
	import { api } from '$lib/api';
	import { formatDuration, formatDurationShort } from '$lib/duration';
	import type { Profile } from '$lib/types';
	import { onMount } from 'svelte';

	let profile = $state<Profile | null>(null);
	let error = $state<string | null>(null);

	onMount(() => {
		load();
	});

	async function load() {
		const username = page.params.username;
		if (!username) {
			error = 'Profile not found';
			return;
		}
		try {
			profile = await api.profile(username);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Profile not found';
		}
	}

	function hostnameOf(url: string): string {
		try {
			return new URL(url).hostname.replace(/^www\./, '');
		} catch {
			return url;
		}
	}
</script>

<svelte:head>
	<title>{profile?.username ?? 'Profile'} — StreakForge</title>
</svelte:head>

<div class="container" style="max-width:640px;padding-top:28px;">
	{#if error}
		<div class="card empty" style="color:var(--red);">{error}</div>
	{:else if !profile}
		<div class="skeleton" style="height:280px;"></div>
	{:else}
		<div class="card" style="text-align:center;margin-bottom:20px;">
			<img
				class="avatar avatar-lg"
				src={profile.avatar_url ?? `https://api.dicebear.com/7.x/identicon/svg?seed=${profile.username}`}
				alt={profile.username}
				style="margin-bottom:10px;"
			/>
			<h1 style="margin:0;font-size:26px;letter-spacing:0.02em;">{profile.display_name ?? profile.username}</h1>
			<p style="color:var(--text-dim);font-size:14px;margin:4px 0 16px;">@{profile.username}</p>
			{#if profile.social_url}
				<p style="margin:0 0 16px;">
					<a
						class="social-link"
						href={profile.social_url}
						target="_blank"
						rel="noopener noreferrer"
					>
						↗ {hostnameOf(profile.social_url)}
					</a>
				</p>
			{/if}
			<div class="stat-grid">
				<div class="stat-card">
					<div class="label">Current streak</div>
					<div class="value red">{profile.streak}d</div>
				</div>
				<div class="stat-card">
					<div class="label">Longest</div>
					<div class="value gold">{profile.longest_streak}d</div>
				</div>
				<div class="stat-card">
					<div class="label">Today</div>
					<div class="value green">{profile.today_count}</div>
				</div>
				<div class="stat-card">
					<div class="label">This week</div>
					<div class="value">{profile.week_count}</div>
				</div>
				<div class="stat-card">
					<div class="label">All-time</div>
					<div class="value">{profile.alltime_count}</div>
				</div>
				<div class="stat-card">
					<div class="label">Trust</div>
					<div class="value gold">TL{profile.trust_level}</div>
				</div>
				<div class="stat-card">
					<div class="label">Verified cage</div>
					<div class="value green">{formatDurationShort(profile.verified_cage_secs)}</div>
				</div>
			</div>
			<!-- The numbers block has to be honest about what it does not show: a missing
			     average is "not shared", never a zero, and the count of reports is published
			     because the average without its sample size is a headline. -->
			<p style="color:var(--text-dim);font-size:12px;margin-top:14px;line-height:1.6;">
				{#if profile.time_to_cum_avg_secs !== null}
					Time to cum: <b style="color:var(--text);">{formatDuration(profile.time_to_cum_avg_secs)}</b>
					over {profile.time_to_cum_samples} report{profile.time_to_cum_samples === 1 ? '' : 's'} —
					shared by choice.
				{:else if profile.time_to_cum_samples > 0}
					Time to cum: not shared ({profile.time_to_cum_samples} report{profile.time_to_cum_samples === 1 ? '' : 's'} kept
					private).
				{:else}
					No durations reported.
				{/if}
			</p>
			<p style="color:var(--text-dim);font-size:12px;margin-top:14px;">
				Forging since {new Date(profile.created_at).toLocaleDateString()}
			</p>
		</div>
	{/if}
</div>
