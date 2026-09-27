# 2026-09-28 — StreakForge: reviewing `wip/recover-2026-08-work`, and making the frontend runnable

## The decision this session had to make

The 2026-09-26 audit left one open item:

> **streakforge** `wip/recover-2026-08-work` — recovered and committed, **never
> reviewed or merged**. Needs a decision on whether the 2026-08 work is
> actually wanted.

Two things were tangled in that branch, and separating them was most of the work.

**Commit `98d61a8`** (hirrolot19, 2026-08-13, on `denial-retheme`) is finished,
authored work: the `denial` log kind, `chastity lock` tracking, weighted
leaderboards, four backend tests, a manifesto rewrite, and three design docs. It
reads as deliberate and complete.

**Commit `bc08d5b`** (Hermes, 2026-09-26) is a *recovery*, not a feature. The
git index had 77 files staged as deleted while present on disk — the signature
of an interrupted `git rm -r --cached`. `HEAD` still held them, so `git reset`
restored the index with no working-tree touch. Anyone committing from that index
would have destroyed 12,543 lines.

**Decision: the work is wanted, and the recovery is already correct.** The
recovered content is not speculative — it is the state of the working tree on
2026-08-14, which means it is what the author was in the middle of building.
Nothing in it is half-finished scaffolding. It merges.

What the branch needed was not a decision about intent but a **review**, because
the recovery had never been built, tested, or run. That is what this document
records.

## What was already good

The backend is honest. This is worth saying plainly because ThreadLight, reviewed
the same week, turned out to have a test suite that had never passed.

- `cargo build` — clean, 33.9s cold.
- `cargo test --test integration` — **18 passed, 0 failed**, against a database
  dropped to zero tables, on the first attempt.
- Repeated three times: 18/18 every time, 2.5-3.1s.

The suite documents its own precondition in the file header
(`require a local Postgres (streakforge_test db)`), which is why it took one
`CREATE DATABASE` and no archaeology. It applies its migrations itself. Fixtures
are per-test unique. The `OnceCell` init is a proper barrier. Test passwords are
hashed at a low cost, unlike ThreadLight.

That the two sibling projects differ this much is the useful finding: the
scraped-up 2026-08 work is in better shape than the 2026-07 work.

## What was broken: the frontend could not be installed

    $ npm ci
    npm error path .../node_modules/vite-imagetools/node_modules/sharp
    npm error command failed
    > sharp@0.34.5 build
    > sharp: Attempting to build from source via node-gyp
    > sharp: Please add node-addon-api to your dependencies

`sharp@0.34.5` has no prebuilt binary for linux-x64 / node 26, and its source
build fails. `npm ci` aborts, so `node_modules` is never populated — which means
`npm run build`, `npm test` and `npm run check` all report `command not found`
and the entire frontend looks unverified.

It was not always broken; it would have started failing when sharp's prebuilds
stopped covering this platform. Nothing in the repo recorded that.

### The cause was a plugin doing nothing

`@sveltejs/enhanced-img` is a direct devDependency, registered in
`vite.config.ts` as `enhancedImages()`. It pulls in
`vite-imagetools` -> `sharp`.

**No component uses `<enhanced:img>`.** `grep -rn 'enhanced:' src/` returns
nothing. The plugin also cannot help: the only three `<img>` tags in the app
take *runtime* URLs —

    src={item.avatar_url ?? `https://api.dicebear.com/7.x/identicon/svg?seed=${item.username}`}

— so there is no build-time asset for it to optimize. It was a build-time native
dependency doing zero work and blocking every frontend command.

Removed: the plugin from `vite.config.ts` and the dependency from
`package.json`. The reasoning is recorded in a comment at the removal site, with
the instruction that if build-time optimization is ever wanted, pin a sharp with
a prebuilt linux-x64 binary rather than relying on a source build.

## Two Svelte 5 deprecations

`npm run check` reported 0 errors and 2 warnings, both in `Navbar.svelte`:

- `<slot />` — deprecated in Svelte 5. The only usage is `<Navbar />` in
  `+layout.svelte`, which passes no children, so the slot rendered nothing. It
  cost a warning and nothing else. Removed, with a comment saying what to use
  instead (`let { children } = $props()` + `{@render children?.()}`) if the
  component ever needs to project content.
- `<div class="nav-spacer" />` — self-closing on a non-void element is ambiguous.
  Changed to `<div ...></div>`.

## Verification

Frontend, after the fixes:

    svelte-check   0 errors, 0 warnings
    vitest         6 files, 47 tests passed
    vite build     built in 7.21s, adapter-static wrote build/

Backend, unchanged by this work but re-verified:

    cargo test --lib          8 passed
    cargo test --test integration   18 passed, x3 consecutive runs

## What is still not decided

`denial-retheme` and `wip/recover-2026-08-work` point at the same two commits
(`98d61a8` + `bc08d5b`); `main` has neither. Merging to `main` is the obvious
next step and is left for a separate, explicit merge rather than folded into a
test-and-fix commit. The two branch names now carry identical content, which is
itself worth cleaning up: one of them should go.
