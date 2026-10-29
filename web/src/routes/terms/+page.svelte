<svelte:head>
	<title>Terms — StreakForge</title>
	<meta
		name="description"
		content="What StreakForge is, what it records, and what it does with it. Adults only."
	/>
</svelte:head>

<div class="container" style="max-width:680px;padding-top:28px;">
	<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 4px;">TERMS AND WHAT THIS IS</h1>
	<p style="color:var(--text-dim);font-size:14px;margin:0 0 20px;">
		Short, and written to be read. If something here surprises you, one of us
		has made a mistake.
	</p>

	<div class="card" style="padding:18px;margin-bottom:12px;">
		<h2 style="font-size:14px;letter-spacing:0.08em;margin:0 0 8px;">ADULTS ONLY</h2>
		<p style="font-size:14px;line-height:1.6;margin:0;">
			The site is for people aged 18 or over. The acknowledgement you may have
			dismissed on your first visit is exactly that — an acknowledgement. It is
			stored in your own browser and nothing on the server checks it. No
			client-side notice can verify an age, and pretending this one does would be
			worse than saying so.
		</p>
	</div>

	<div class="card" style="padding:18px;margin-bottom:12px;">
		<h2 style="font-size:14px;letter-spacing:0.08em;margin:0 0 8px;">WHAT IT IS</h2>
		<p style="font-size:14px;line-height:1.6;margin:0 0 10px;">
			A self-tracking tool for one person's own habits, plus public counters and
			boards. It is not a service, not therapy, not medical advice, and not a
			commitment device with any authority behind it. The dates and counts are
			self-reported: nothing here can be verified and nothing here claims to be.
		</p>
		<p style="font-size:14px;line-height:1.6;margin:0;">
			The numbers are deliberately hard to game for someone who is playing fair —
			see the <a href="/glossary">glossary</a> for how — and completely trivial to
			falsify for someone who is not. That is a design choice, not an oversight.
		</p>
	</div>

	<div class="card" style="padding:18px;margin-bottom:12px;">
		<h2 style="font-size:14px;letter-spacing:0.08em;margin:0 0 8px;">NOTES ARE PRIVATE</h2>
		<p style="font-size:14px;line-height:1.6;margin:0;">
			The note attached to a log is readable only by its author, on their own
			dashboard. It is not published to the activity feed, not shown on your
			profile, and not returned by the public API. The feed shows that a log
			happened, never what was written. If that ever changes it will be opt-in and
			said in advance.
		</p>
	</div>

	<div class="card" style="padding:18px;margin-bottom:12px;">
		<h2 style="font-size:14px;letter-spacing:0.08em;margin:0 0 8px;">WHAT IS PUBLIC</h2>
		<p style="font-size:14px;line-height:1.6;margin:0;">
			Your username, avatar, the counters and the scores are public, along with the
			fact and time of each log. That is what the boards are. Your password is
			stored only as an Argon2 hash and is never shown to anyone, including the
			operator.
		</p>
	</div>

	<div class="card" style="padding:18px;margin-bottom:12px;">
		<h2 style="font-size:14px;letter-spacing:0.08em;margin:0 0 8px;">DELETION AND EXPORT</h2>
		<p style="font-size:14px;line-height:1.6;margin:0;">
			Ask and your account, logs and notes are deleted, or handed to you as a file
			first if you want them. A deleted account's logs leave the boards and the
			counters with it. Requests go through whichever contact route the instance
			publishes; there is no self-service button yet, and this page will say so
			until there is.
		</p>
	</div>

	<div class="card" style="padding:18px;margin-bottom:12px;">
		<h2 style="font-size:14px;letter-spacing:0.08em;margin:0 0 8px;">COOKIES AND CONSENT</h2>
		<p style="font-size:14px;line-height:1.6;margin:0 0 8px;">
			One cookie is essential: the session that keeps you signed in. It is set the
			moment you have a session to keep, and the site does not work without it.
			Analytics and personalisation are optional, off by default, and only turned on
			if you tick them.
		</p>
		<p style="font-size:14px;line-height:1.6;margin:0;">
			Declining is honoured: the counters stay readable, the boards stay readable,
			and nothing is stored on your device beyond the per-tab flag that remembers you
			said no. What you cannot do without accepting is write — post, log, or register
			— because the API refuses that server-side, not just in the browser. Your
			answer is recorded against your account once you have one, so signing in on
			another device does not ask you again. Changing what is collected means a new
			version of this notice, and a new version is a new question.
		</p>
	</div>

	<div class="card" style="padding:18px;">
		<h2 style="font-size:14px;letter-spacing:0.08em;margin:0 0 8px;">THE SHORT VERSION</h2>
		<p style="font-size:14px;line-height:1.6;margin:0;">
			Consenting adults, self-reported numbers, public activity, and confessions you can make private in settings. No
			warranty, no promises about availability, and no interest in your data beyond
			keeping the counters running.
		</p>
	</div>

	<p style="color:var(--text-dim);font-size:12px;margin-top:20px;">
		See also the <a href="/glossary">glossary</a> for the vocabulary and the
		<a href="/doctrine">doctrine</a> for the rules.
	</p>
</div>
