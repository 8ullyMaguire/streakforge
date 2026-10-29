<script lang="ts">
	// PRIVACY — the data rights, written down next to the buttons that exercise them.
	//
	// This page exists because the site holds unusually intimate records (locked
	// time, denials, reported durations) and the terms promised export and erasure
	// before either existed. Every claim here maps to something in the code; the
	// last section says what the site cannot do, which is the part a policy page
	// usually leaves out.
</script>

<svelte:head>
	<title>Privacy — StreakForge</title>
	<meta
		name="description"
		content="What StreakForge stores, how to take it with you, how to have it erased, and what survives a deletion."
	/>
</svelte:head>

<div class="container" style="max-width:720px;padding-top:28px;">
	<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 8px;">PRIVACY</h1>
	<p style="color:var(--text-dim);font-size:13px;margin:0 0 22px;line-height:1.6;">
		Short version: the site keeps what you log, shows what you publish, and can hand the whole
		thing back or delete it on request. There is no third-party analytics script, no advertising
		network, and no email address anywhere in the system.
	</p>

	<h2 style="font-size:16px;letter-spacing:0.08em;margin:22px 0 10px;">WHAT IS STORED</h2>
	<div class="card" style="padding:16px;font-size:13px;line-height:1.65;">
		<ul style="margin:0;padding-left:18px;color:var(--text-dim);">
			<li><b style="color:var(--text);">Account</b> — username, password hash (Argon2id), optional display name, avatar URL and one social URL.</li>
			<li><b style="color:var(--text);">Logs</b> — kind (waste, denial or drill), timestamp, optional confession note (140 characters), optional reported duration.</li>
			<li><b style="color:var(--text);">Locks</b> — when a lock started and ended, and the reason.</li>
			<li><b style="color:var(--text);">Cage checks</b> — the seal code the site issued, what you claimed, and who attested it.</li>
			<li><b style="color:var(--text);">Moderation</b> — flags you filed, and actions taken on content.</li>
			<li><b style="color:var(--text);">Session</b> — a server-side row so a cookie can sign you in; no IP address is kept with it.</li>
			<li><b style="color:var(--text);">Consent record</b> — when you accepted which version of the gate.</li>
		</ul>
	</div>

	<h2 style="font-size:16px;letter-spacing:0.08em;margin:22px 0 10px;">TAKE IT WITH YOU</h2>
	<p style="font-size:13px;color:var(--text-dim);line-height:1.65;">
		<a href="/settings">Settings → Your data</a> downloads one JSON file containing the account,
		every log, lock, cage check, attestation you gave, flag you filed, moderation action you took,
		and the audit events attached to you. It is built by one SQL statement, so a table the export
		forgets is a test that fails rather than a promise that quietly weakens.
	</p>

	<h2 style="font-size:16px;letter-spacing:0.08em;margin:22px 0 10px;">HAVE IT ERASED</h2>
	<p style="font-size:13px;color:var(--text-dim);line-height:1.65;">
		<a href="/settings">Settings → Your data → Erase account</a> requires your password and the word
		<code>DELETE</code> typed by hand. It deletes the profile — and, by the foreign keys, every log,
		lock, cage check, attestation and flag that belongs to it — clears your sessions, and empties the
		detail of your audit rows. Two things deliberately survive: an anonymous row saying that an
		erasure happened (with counts, no name), and any moderation decision you took about someone
		else's content, with its actor set to null, because that record is about their content rather
		than your account.
	</p>

	<h2 style="font-size:16px;letter-spacing:0.08em;margin:22px 0 10px;">CORRECT IT</h2>
	<p style="font-size:13px;color:var(--text-dim);line-height:1.65;">
		Profile fields are editable on the settings page. For 15 minutes after writing a log you can set
		or clear its duration and undo it entirely; a backdated (catch-up) log measures that window from
		when it was <i>written</i>, not from the day it describes. Older rows can be retracted by a
		moderator, not edited — the site would rather hide a row than rewrite someone's history.
	</p>

	<h2 style="font-size:16px;letter-spacing:0.08em;margin:22px 0 10px;">WHAT IS PUBLIC</h2>
	<p style="font-size:13px;color:var(--text-dim);line-height:1.65;">
		Public: username, display name, avatar, social URL, your score, streak, board positions, and the
		kinds of log plus the date of each entry. Confessions are public by default and can be turned off
		in settings; with that off your rows leave the feed entirely while your own history and score are
		untouched. Reported durations are aggregated; a per-person average appears on the metrics page once
		five are reported. Locks show as locked or unlocked, never with what is worn.
	</p>

	<h2 style="font-size:16px;letter-spacing:0.08em;margin:22px 0 10px;">HOW LONG</h2>
	<p style="font-size:13px;color:var(--text-dim);line-height:1.65;">
		Content lives until you delete it or the account. Sessions expire after 30 days. Failed sign-in
		attempts are kept for 30 days, then purged. Folded guest tombstones are kept for a year so a
		converted account can still answer "where did my history go", then purged. Audit rows that are not
		a consent record or an erasure are purged after a year. Nothing is kept "for analytics".
	</p>

	<h2 style="font-size:16px;letter-spacing:0.08em;margin:22px 0 10px;">WHAT THIS SITE CANNOT DO</h2>
	<div class="card" style="padding:16px;font-size:13px;line-height:1.65;color:var(--text-dim);">
		<p style="margin:0 0 8px;">
			<b style="color:var(--text);">No email means no reset and no receipt.</b> There is no address on
			file, so a lost password is lost, and an erasure happens without a confirmation email.
		</p>
		<p style="margin:0 0 8px;">
			<b style="color:var(--text);">Nothing is end-to-end encrypted.</b> The operator can read the
			database. Self-reported metrics are exactly that — the cage was designed so that a claim needs
			peers instead of a photo, not so that the claim is provable.
		</p>
		<p style="margin:0;">
			<b style="color:var(--text);">Deletion is real, which means it is irreversible.</b> There is no
			trash, no 7-day undo and no backup restore of one account. Take the export first.
		</p>
	</div>

	<p style="color:var(--text-dim);font-size:12px;margin-top:20px;">
		See also the <a href="/terms">terms</a> and the <a href="/glossary">glossary</a>.
	</p>
</div>
