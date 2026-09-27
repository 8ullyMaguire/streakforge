# StreakForge — Feature Ideas

Written 2026-09-28, against the tree as it actually stands today (migrations
0001–0009, 9 routes, `docs/` plan already executed). This is a *second* pass:
`BRAINSTORM_POPULARITY.md` proposed the growth strategy and a lot of it shipped.
What follows is grounded in the current schema and the gaps that are still real,
not a re-run of that list.

**How to read the tiers.** T1 is what I'd build next. T2 is worth doing once T1
is solid. T3 is speculative. Effort is rough: S = under a day, M = a few days,
L = more than a week.

---

## Where the product actually stands

Shipped and verified: daily UTC cadence (one log per kind per day), the
exclusivity rule (a waste and a denial can't both count on the same day), the
Whiteboi Devotion Index with tiered multipliers, 30-day score decay with a
separate lifetime total, the chastity lock, drill, feed, profile, and doctrine.

The gaps that matter fall into three groups:

1. **Everything is single-player.** A whiteboi's only audience is himself. The
   social machinery is the single largest unrealised opportunity, and it is the
   one the original brainstorm got right.
2. **The site is read-only to the outside world.** Nothing is embeddable,
   nothing is shareable, no one can be pointed at a user's streak from elsewhere.
3. **There is no re-engagement.** No notifications, no email (deliberately), no
   reminders. A streak product that only speaks to you when you open it is a
   diary, not a habit.

---

## T1 — the ones I'd build next

### 1.1 Keyholder link
The biggest missing thing versus the real-life kink, and the only idea here that
creates a genuine two-sided network effect: whitebois recruit their keyholders,
keyholders recruit whitebois, and the site becomes somewhere you *bring someone*.

- v1 is small: a `keyholder` field on the profile naming another user, plus an
  unlock *request* that the keyholder approves or denies. A locked user who
  cannot end their own lock without someone else's approval has an actual stake.
- Schema: `keyholder_id uuid references profiles(id)` on the profile, and a
  small `unlock_requests` table (id, user_id, reason, status, decided_at).
- Watch the failure mode: a user naming a stranger who never logs in is locked
  out forever. Needs a stated escape hatch (N days without a keyholder response
  reverts to self-unlock, and says so plainly).
- Effort: M. This is the highest-leverage item in the document.

### 1.2 Shareable cards
Every active user becomes a billboard. `wlw.grok.me` has nothing like it.

- Server-rendered PNG or SVG: username, current lock duration, today's score,
  leaderboard rank, the doctrine's dark/red/gold palette. Watermarked with the
  domain.
- Share target: Bluesky first, then X. Bluesky's API is the friendliest for
  unattended posting and it is where this audience already is — there is
  already a `bnwo-feed.polarisocial.xyz` custom feed in the doctrine's socials
  doc, so the audience overlap is real and reachable.
- Effort: M (card rendering), S (posting).

### 1.3 Proof that cannot be screenshotted
A screenshot of "locked 14d" is worth nothing. A signed one is worth something.

- A `proof_key` per lock session: a short deterministic string derived from
  `lock_start + user_id + a server secret`, rendered on the profile and
  verifiable by anyone with a copy of the algorithm.
- This turns a claim into a checkable fact, and makes "bet you can't stay locked
  longer" a real challenge rather than a boast.
- Effort: S, and it makes 1.2 substantially more credible.

### 1.4 Public ledger for wlw.grok.me
A direct competitor is named in the repo. Rather than replicate it, be legible
where it isn't.

- A public, per-user, paginated list of log timestamps — dates and kinds, no
  notes, no private text. The point is verifiability: anyone can confirm a
  streak is real without trusting a screenshot.
- Free by construction, since it is a read-only view of data the site already
  has, and it is the thing that makes the proof key in 1.3 worth anything.
- Effort: S, given the schema is already there.

### 1.5 Fix the two things I found while writing this

Not features, but they are the reason two adjacent ideas are currently unsafe:

- **`/api/feed` uses `select *`** (`api.rs:789`). That is the exact pattern that
  500'd both leaderboards twice already this month — once for a `numeric` it
  could not decode into `f64`, once for an `INT4` where `i64` was expected. The
  feed is currently one function-signature change away from another 500, and it
  is the one endpoint with the most anonymous traffic. Enumerate the columns
  like `get_leaderboard` already does, or pin it with a test.
- **Log notes are public.** `note` is free text up to 140 chars and
  `activity_feed` returns it to unauthenticated callers. Someone will eventually
  write something there they did not mean to publish. Either drop notes from the
  anonymous feed, gate the feed behind auth, or add a per-user "public feed"
  toggle. Worth deciding before the site is shared anywhere, not after.
- Effort: S each, and both are cheaper than the outage they prevent.

---

## T2 — worth doing once T1 is solid

### 2.1 PWA and notifications
The manifest exists and is linked from `app.html`, but there is no service
worker, so nothing is actually installable and nothing can notify.

- A service worker plus a real offline shell, so the app opens instantly and
  survives a bad connection on a phone.
- Web push for one notification a day: the countdown to the next UTC midnight,
  phrased as the site already phrases it. Opt-in, one a day, no nagging.
- Effort: M. Notifications are the entire difference between a habit product and
  a diary, and the daily UTC cadence is already perfectly shaped for it — one
  push at a fixed time is trivially reliable.

### 2.2 Accountability pairs
Two users, both must log within 24h or the pair's streak breaks. Friend pressure
beats app pressure.

- Cheap to build once 1.1 exists, and the pair mechanic reuses the exclusivity
  and cadence logic unchanged.
- Watch for a bad edge: a pair where one member goes quiet punishes the other.
  Decide whether the pair breaks, the remaining member is exempted, or nothing
  happens — and make it a setting.

### 2.3 Seasonal events
Locktober and No Nut November are already referenced in the manifestos, so the
audience arrives already primed. `lock_sessions` is the right substrate.

- An event is a named date range with its own board and a survivor badge. The
  complication is the interaction with the 30-day decay: an event longer than 30
  days would wipe a score mid-event, so events need an explicit exemption or a
  shorter horizon. Decide this before building, not after.

### 2.4 Doctrine as a funnel
The ten manifestos are the best content the site has and none of them are
indexable as anything but a JSON blob behind `/api/doctrine`.

- Per-document public pages with real titles and descriptions, a "next read"
  chain from the starting guide onward, and per-page meta.
- This is the one genuinely passive acquisition channel available, and it
  compounds without any product work.

### 2.5 Embeddable streak badge
An `<iframe>` widget rendering a user's public streak, for blogs and link-in-bio
pages. Every external profile becomes a StreakForge billboard.

- Only safe once 1.4 decides what is public. Build the visibility rules first;
  the embed is the easy half.
- Effort: S for the widget, and it depends entirely on 1.1/1.4's decisions.

---

## T3 — speculative, listed so they are not lost

- **Public API.** Read-only endpoints for lock state and stats so the community
  can build bots and dashboards. Free distribution, but it is an obligation
  once published, and the privacy question in 1.5 should be settled first.
- **Referral loop.** Invite links, both parties get a small one-time bonus.
  Thematic, effective, and trivial — which is also why it is a good late
  addition rather than an early one.
- **Time trials.** `manifestos/08` already specifies the levels. A timer page
  with a "fastest loser" board. Premature is celebrated on theme, so it fits
  without irony.
- **Affirmation syndication.** A daily card, posted unattended via cron. Cheap
  daily content that reinforces the drill habit.

---

## What I would not build

- **Ads, or anything that dilutes the aesthetic.** The dark/red/gold look is the
  brand, and trust in this audience is the product.
- **Mandatory email, phone, or identity verification.** It would kill the
  anonymous-fetish audience, which is most of the audience.
- **A paywall on the counter or the board.** Those stay free or they stop being
  the reason anyone shows up.
- **Anything that rewards wasting over denying.** The board's entire story is
  that the top of it is the people denying hardest. A change that inverts that
  incentive is not a feature.
- **A "clean" rebrand for a mainstream audience.** The kink is the product; the
  word-of-mouth depends on it staying specific.

---

## A note on the 1.5 items

They are filed as features because they are small, but they are the only two
items here that are *bugs* wearing a feature's clothes. The `select *` in the
feed has the same shape as two 500s already fixed this month, and the public
notes are a privacy exposure that only becomes visible once someone is actually
sharing the site. Both are cheaper to fix now than to explain later.
