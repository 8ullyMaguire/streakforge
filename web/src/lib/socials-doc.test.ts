// The socials doc is content the site serves, and it carries two things the owner
// asked for by name: EVERY site in the network, and a working way to ask for
// changes. Both are checkable against the rendered HTML, which is what a reader
// gets -- the source being right is not the claim.
//
// This also guards a formatting trap in this codebase's own tiny renderer: it
// resolves bare URLs BEFORE bold, so `**https://host/path**` can end up with the
// asterisks inside the href. That renders as a link to nothing.
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { renderMarkdown } from './markdown';

// vitest runs with the web/ directory as cwd, and `import.meta.url` is an http
// URL under jsdom, so the doc is resolved from the working directory and read as
// a FILE. If this path is wrong the read throws -- it does not quietly render an
// empty document, which would make the completeness assertions below vacuous.
const DOC = resolve(process.cwd(), '../manifestos/09_socials.md');
const src = readFileSync(DOC, 'utf8');
const html = renderMarkdown(src);

/// Every site the owner listed, by hostname. Two of them are named but NOT linked, because
/// linking them would be a lie the reader pays for: gravity.polarisocial.xyz has no DNS record
/// (NXDOMAIN, checked against Cloudflare's resolver) and leftist.polarisocial.xyz answers 502.
/// Both are still NAMED -- the owner asked for the list of their sites, and the intention is
/// part of that -- which is why completeness is asserted on the text and linking separately.
const SITES = [
	'streakforge.polarisocial.xyz',
	'whitebois.polarisocial.xyz',
	'blackrep.polarisocial.xyz',
	'gravity.polarisocial.xyz',
	'concord.polarisocial.xyz',
	'lorehaven.polarisocial.xyz',
	'ficnexus.polarisocial.xyz',
	'kindred.polarisocial.xyz',
	'stashdb.polarisocial.xyz',
	'piefed.polarisocial.xyz',
	'bnwo-feed.polarisocial.xyz',
	'news.polarisocial.xyz',
	'icecast.polarisocial.xyz',
	'git.polarisocial.xyz',
	'leftist.polarisocial.xyz',
	'castopod.polarisocial.xyz'
];

/** Every href in the rendered doc, in order. */
const hrefs = (): string[] => [...html.matchAll(/href="([^"]+)"/g)].map((m) => m[1]);

/// The one host with no DNS record at all. leftist is NOT in this group: its hostname
/// resolves and the entry says the site is not up yet, so its link is a real address that will
/// work when the app lands. gravity.polarisocial.xyz is NXDOMAIN -- linking that is a dead
/// letter, so the entry names it as text instead.
const UNRESOLVING = 'gravity.polarisocial.xyz';
const LIVE_SITES = SITES.filter((h) => h !== UNRESOLVING);

describe('manifestos/09_socials.md, rendered', () => {
	it('names every site in the network', () => {
		expect(SITES.length, 'the list the owner gave has sixteen sites').toBe(16);
		const missing = SITES.filter((h) => !html.includes(h));
		expect(missing, `sites missing from the doctrine doc: ${missing.join(', ')}`).toEqual([]);
	});

	it('addresses each of them, and every URL in the doc is a link', () => {
		// The count assertion first: an absence assertion over an empty document
		// is vacuously true, which is how "the list is complete" passes on a doc
		// that failed to load at all.
		const all = hrefs();
		expect(all.length, 'the doc must render links at all').toBeGreaterThan(10);
		// One assertion per host: the bare-URL pass keeps whatever the author
		// wrote, so the list is written without trailing slashes and a host with
		// one elsewhere in the doc would still satisfy a prefix match.
		for (const h of LIVE_SITES) {
			expect(all, `${h} must be an href, not just text`).toContain(`https://${h}`);
		}
		// Named, deliberately NOT linked, and the entry says why. Linking a host
		// that does not answer is the failure this whole test file exists to stop.
		for (const h of [UNRESOLVING]) {
			expect(html, `${h} must still be named`).toContain(h);
			expect(all, `${h} has no DNS record, so it must not be a link`).not.toContain(`https://${h}`);
		}
	});

	it('links the Concord project with nothing stray inside the href', () => {
		// Written as an explicit `[text](url)` in the doc, NOT as `**url**`: a bare
		// URL wrapped in bold does not linkify at all in this renderer (the marker
		// sits between the start of the line and the URL), so the CTA would be
		// inert bold text. This asserts the shape that works.
		const all = hrefs();
		expect(all).toContain('https://concord.polarisocial.xyz/projects/streakforge');
		// `*` is a markdown marker and `<` means a tag ended up inside an
		// attribute; `_` is NOT checked, because it is ordinary URL content (the
		// rule34 tag lists below are full of it).
		const bad = all.filter((h) => /[*<]/.test(h));
		expect(bad, `hrefs carrying markdown or tags: ${bad.slice(0, 2).join(', ')}`).toEqual([]);
	});

	it('keeps the long favourite links single-escaped, not double', () => {
		// Published for weeks with href="...?page=post&amp;amp;s=list&amp;amp;tags=":
		// the browser reads that as a parameter literally named `amp;s`, so the
		// two rule34 links went to a page with no filters. Found by this test.
		const rule34 = hrefs().filter((h) => h.includes('rule34'));
		expect(rule34.length, 'the doc has two rule34 favourite links').toBe(2);
		for (const h of rule34) {
			expect(h, 'a double-escaped href is a broken link').not.toContain('amp;amp;');
			expect(h).toContain('&amp;s=list');
		}
	});

	it('says what can be done there: complaints, features, and the arena', () => {
		const text = html.replace(/<[^>]+>/g, ' ');
		for (const phrase of ['File a complaint', 'File a feature', 'Vote in the arena']) {
			expect(text, `the doc must tell a reader they can ${phrase}`).toContain(phrase);
		}
	});

	it('keeps the wrapped sentences INSIDE their list items', () => {
		// Every entry here is prose wrapped at ~90 columns, so the continuation
		// lines are the thing that can break: the renderer used to close the list
		// and emit the rest of the sentence as a <p> after it, which reads as a
		// paragraph stuck between bullets. The tell is a continuation fragment
		// standing alone in a paragraph.
		const strays = [...html.matchAll(/<p>([^<]{0,80})/g)].map((m) => m[1]);
		// Fragments that are list-item continuations in the source: if one of them
		// starts a paragraph, that item was split.
		const broken = strays.filter((s) => /^(currency: a feature|proposals go head|that is deliberate)/.test(s));
		expect(broken, `continuation lines rendered outside the list: ${broken.join(' | ')}`).toEqual([]);
		// ...and the item count, so an absence assertion over a doc that failed to
		// load cannot pass.
		expect([...html.matchAll(/<li>/g)].length).toBeGreaterThanOrEqual(30);
	});

	it('is honest about the four entries that are not what they say', () => {
		// Listing a dead or misrouted site as if it worked is the one thing this doc must not
		// do. Each of these was measured: leftist answers 502, castopod serves the WhiteBois
		// app, gravity.polarisocial.xyz is NXDOMAIN, and icecast.polarisocial.xyz serves the
		// GRAVITY app rather than a stream.
		const text = html.replace(/<[^>]+>/g, ' ');
		expect(text).toMatch(/leftist[\s\S]{0,200}not up yet/i);
		expect(text).toMatch(/castopod[\s\S]{0,200}not up yet/i);
		expect(text).toMatch(/gravity[\s\S]{0,240}no DNS record/i);
		expect(text).toMatch(/icecast[\s\S]{0,240}serves the GRAVITY app/i);
	});
});
