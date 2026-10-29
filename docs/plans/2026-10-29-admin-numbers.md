# Plan — the product numbers, and closing the roadmap

Date: 2026-10-29 (late) · Repo: `~/code-local/rust/streakforge` · Owner, verbatim:

> "#123 a TL4 /admin page with product numbers (weekly active humans, guest→account conversion,
> retention) implement these and dismiss everything else in the roadmap; test, verify, etc as you go;
> when done update docs (readme, roadmap, etc) if needed, commit, push, re-deploy, etc"

So: build #123, and close the roadmap — nothing else from it gets built, and the record says so
plainly rather than leaving 140 ideas looking like a queue.

## 1. Decisions

| question | decision | why |
|---|---|---|
| Where does the arithmetic live? | three SQL functions (0022), not handler queries | a metric is a definition; written twice it drifts, and psql, tests and page must agree |
| Who may read it? | `profiles.is_leader` only (trust level 4) | the same bit that can hide anyone's post; there is no moderator tier to leak it to |
| Are reads audited? | no, and the code says why | `audit_log` records actions; a row per page view would be a browsing log of the leader, not a protection for members. What protects members is that the payload is counts |
| Is any member named? | no | the questions are "is this growing" and "do anonymous visitors come back"; neither needs a person |
| What is "active"? | ≥1 log of any kind in the window, members only | the question is whether someone used the site, not what they logged |
| When is a guest "converted"? | counted in the week the **guest appeared**, not the week it registered | the funnel question is "of those who tried anonymously, how many came back with a name"; the other choice flatters the current week |
| Future retention windows? | `null`, printed "—" | a 0 there reads as "everyone who joined on Monday left", which the data has not said |
| Median logs per active member? | `percentile_cont(0.5)` over members | an average over logs lets one heavy logger define the typical member |
| Time windows? | 12 weekly rows (current week included, incomplete), 8 cohorts, W1..W4 | enough to see a trend, short enough to fit one screen |

## 2. Files

- `backend/migrations/0022_admin_numbers.sql` — `admin_overview()`, `admin_weekly(p_weeks)`,
  `admin_retention(p_weeks)`, each with its definitions in comments.
- `backend/src/admin.rs` — the TL4 gate, the casts (`bigint`/`float8` on every numeric column), the
  shipped definitions (`notes[]`).
- `backend/src/router.rs`, `backend/src/lib.rs`.
- `web/src/routes/admin/+page.svelte` — overview cards, the funnel, the weekly bars, the cohort grid,
  the definitions in a `<details>`, and the refusal card for everyone else.
- `web/src/routes/moderation/+page.svelte` — a door to `/admin`, leaders only.
- `web/src/lib/{types,api}.ts`, `scripts/deploy.sh` (health check for `/admin`).

## 3. Verification

1. **The arithmetic, by hand.** `cargo test --test integration the_admin_numbers` builds a cohort by
   hand — three members in the week of 2026-09-28, logs in W1 (two), W2 (one), W3 (none), W4 (one),
   two guests in the week of 2026-10-19 with one converted — and asserts `size 3, w1 2, w2 1, w3 0,
   w4 1`, rate `0.5`, and that the current cohort's windows are `null`.
2. **Guests never inflate "active".** Same test: a guest logs and `active_7d` stays 1.
3. **The gate and the decode.** `cargo test --test routes the_numbers_page` — 401 anonymous, 403 for a
   member with the word "leader" in the message, 200 for a leader with every numeric field an `i64`
   or `f64`, 12 weekly rows, 8 cohorts, the notes array, and `conversion_rate` null with no guests.
4. **Suites**: 148 backend (22 unit · 11 consent · 14 guest · 67 integration · 27 routes · 7 edge),
   145 vitest, svelte-check 0/0, `vite build`.
5. **Live**, against production: anonymous 401; a fresh member 403 ("The numbers page belongs to the
   site's leader…"); with a temporary leader grant 200 and the numbers cross-checked against SQL —
   `members 74`, `active_7d 3`, `guests 0`, `conversion_rate null`, 12 weekly rows,
   8 cohorts, 6 notes.
6. **In a browser**: `/admin` as the leader renders the cards, the funnel ("0 guests ever … —"), the
   weekly table with bars, the retention grid with "—" for future windows and real percentages for the
   Oct-5 cohort (`3 38%`, `1 13%`, `0 0%`); after the leader bit is removed, the same page shows the
   refusal card instead. The probe account was deleted afterwards (73 members, 0 leaders).

## 4. What was dismissed, and what that means

Everything not already shipped from `docs/100_IDEAS.md` and `docs/FEATURE_IDEAS.md`, and GS-5 from
`docs/ROADMAP-guest-trust-governance.md`. Those files now carry a closure banner: they are records of
what was considered, not backlogs. Nothing in them is scheduled, and nothing there should be picked up
"because it was on the list" — it reopens by asking the owner.

Two specific things I did **not** build, and would have built next if the roadmap had stayed open:
**#121** a safety/aftercare page (content, not code — the one omission that turns a kink tracker into a
liability) and **#123's follow-on** a per-cohort drill-down. Both are dismissed with the rest.
