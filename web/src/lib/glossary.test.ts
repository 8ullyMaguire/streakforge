// The glossary as CONTENT (item 12).
//
// A list of definitions on a page is easy to let rot: a term gets renamed in the
// code and the page keeps explaining the old one. So the glossary is an exported
// array and these tests assert it is complete and self-consistent — and that the
// page's own internal links point at routes that exist, because a glossary that
// links to a 404 is worse than one with no links.
import { describe, it, expect } from 'vitest';
import { readFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';
import { TERMS } from './glossary';

const ROUTES = join(__dirname, '..', 'routes');

describe('the glossary is complete', () => {
	it('defines every term the site actually uses', () => {
		const names = TERMS.map((t) => t.term);
		// The nouns a newcomer meets in their first minute, each of which is a
		// rule rather than a word: WLW/WLD are kinds, Devotion Index and
		// Multiplier are the score, Decay and Lifetime are its horizon, and the
		// rest are what the pages are called.
		for (const required of [
			'WLW',
			'WLD',
			'Exclusivity',
			'Devotion Index',
			'Multiplier',
			'Active streak',
			'Game bonus',
			'Decay',
			'Lifetime total',
			'Waste budget',
			'Denial cadence',
			'Drill',
			'Lock',
			'Streak',
			'Note'
		]) {
			expect(names, `the glossary is missing "${required}"`).toContain(required);
		}
	});

	it('never defines a term twice', () => {
		const names = TERMS.map((t) => t.term);
		expect(new Set(names).size).toBe(names.length);
	});

	it('gives every term a definition and says where it is enforced', () => {
		for (const t of TERMS) {
			expect(t.definition.length, `${t.term} has no definition`).toBeGreaterThan(30);
			expect(t.enforced.length, `${t.term} names no enforcement point`).toBeGreaterThan(3);
		}
	});

	it('says that notes are private, because that is what changed', () => {
		// The single most load-bearing definition on the page: users were
		// publishing notes without being told (see docs/100_IDEAS.md #01).
		const note = TERMS.find((t) => t.term === 'Note');
		expect(note?.definition.toLowerCase()).toContain('private');
	});
});

describe('the glossary page', () => {
	const src = readFileSync(join(ROUTES, 'glossary', '+page.svelte'), 'utf8');

	it('renders the terms from the module rather than a copy', () => {
		// Two lists would drift; the page must read TERMS.
		expect(src).toContain("from '$lib/glossary'");
		expect(src).toContain('{#each TERMS');
	});

	it('links only to routes that exist', () => {
		const hrefs = [...src.matchAll(/href="(\/[a-z0-9/-]*)"/g)].map((m) => m[1]);
		expect(hrefs.length).toBeGreaterThan(0);
		for (const href of hrefs) {
			const dir = join(ROUTES, href.replace(/^\//, '').split('/')[0]);
			expect(existsSync(join(dir, '+page.svelte')), `glossary links to ${href}, which does not exist`).toBe(
				true
			);
		}
	});
});
