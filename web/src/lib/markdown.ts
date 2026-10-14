// Tiny markdown renderer — enough for the manifesto docs.
// Handles: headings, bold, italic, links, inline code, code blocks, lists,
// blockquotes, paragraphs, hr. No HTML passthrough (escape everything first).

export function renderMarkdown(src: string): string {
	const lines = src.replace(/\r\n/g, '\n').split('\n');
	const out: string[] = [];
	let inCode = false;
	let inList = false;
	let codeBuf: string[] = [];
	/** Index in `out` of the last <li>, or -1. A wrapped line appends to it. */
	let openItem = -1;

	function closeList() {
		if (inList) {
			out.push('</ul>');
			inList = false;
			openItem = -1;
		}
	}

	for (let i = 0; i < lines.length; i++) {
		const line = lines[i];
		const trimmed = line.trim();

		// code fence
		if (trimmed.startsWith('```')) {
			if (inCode) {
				out.push(`<pre><code>${escapeHtml(codeBuf.join('\n'))}</code></pre>`);
				codeBuf = [];
				inCode = false;
			} else {
				closeList();
				inCode = true;
			}
			continue;
		}
		if (inCode) {
			codeBuf.push(line);
			continue;
		}

		if (trimmed === '') {
			closeList();
			continue;
		}

		// hr
		if (/^---+\s*$/.test(trimmed) || /^\*\*\*+\s*$/.test(trimmed)) {
			closeList();
			out.push('<hr />');
			continue;
		}

		// headings
		const h = /^(#{1,6})\s+(.*)$/.exec(trimmed);
		if (h) {
			closeList();
			const level = h[1].length;
			out.push(`<h${level}>${inline(escapeHtml(h[2]))}</h${level}>`);
			continue;
		}

		// blockquote
		if (trimmed.startsWith('>')) {
			closeList();
			out.push(`<blockquote>${inline(escapeHtml(trimmed.slice(1).trim()))}</blockquote>`);
			continue;
		}

		// list item
		const li = /^[-*+]\s+(.*)$/.exec(trimmed) || /^\d+[.)]\s+(.*)$/.exec(trimmed);
		if (li) {
			if (!inList) {
				out.push('<ul>');
				inList = true;
			}
			out.push(`<li>${inline(escapeHtml(li[1]))}</li>`);
			openItem = out.length - 1;
			continue;
		}

		// A wrapped line CONTINUES what came before it: the open list item if there
	// is one, otherwise the open paragraph.
		//
		// WHY (lazy continuation): prose is wrapped at ~90 columns, so a sentence
		// longer than the wrap point arrives here as a bare line. Without this the
		// block closed and each wrapped line became its OWN paragraph -- the socials
		// doc rendered as fourteen fragment paragraphs after the bullets, and its
		// prose as a paragraph per line. Markdown says consecutive text lines are
		// one block; this is that rule for the small subset of markdown these
		// doctrine docs use.
		//
		// Measured across the eleven manifesto docs before the change: only the
		// socials doc has continuation lines, so no existing page renders
		// differently (SPEC §10.5).
		const text = inline(escapeHtml(trimmed));
		if (inList && openItem >= 0) {
			out[openItem] = `${out[openItem].slice(0, -'</li>'.length).trimEnd()} ${text}</li>`;
			continue;
		}

		// A bare text line is its own paragraph.
		//
		// DELIBERATELY NOT lazy continuation for paragraphs, though Markdown does
		// that: these docs put one paragraph per line, so every line that follows
		// another line is a deliberate break -- a bold label above its description,
		// a link above "Comment there if you're interested". Joining them changed
		// two published pages (01_starting_guide.md) as a side effect of a change
		// requested for a different one. Wrap prose as one line per paragraph, and
		// if that ever becomes the wrong trade, this is the place to revisit it.
		closeList();
		out.push(`<p>${text}</p>`);
	}
	closeList();
	return out.join('\n');
}

function escapeHtml(s: string): string {
	return s
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;');
}

function inline(s: string): string {
	// links [text](url)
	s = s.replace(
		/\[([^\]]+)\]\(([^)\s]+)\)/g,
		(m, text: string, url: string) => {
			const safe = url.startsWith('http') || url.startsWith('https') || url.startsWith('/')
				? url
				: '#';
			// NOT escaped again: `s` was escaped by the caller, so both the text and
			// the URL are already entity-encoded. Escaping twice produced
			// href="...?page=post&amp;amp;s=list" -- a link whose query string is
			// literally `amp;s`, which is a broken link that looks correct on the
			// page. The two rule34 "Favorites" links in 09_socials.md were broken
			// this way, in a doc that has been published for weeks. The bare-URL
			// pass below already emits as-is for exactly this reason; pin it in
			// markdown.test.ts so the next edit does not reintroduce it.
			return `<a href="${safe}" target="_blank" rel="noopener">${text}</a>`;
		}
	);
	// bare URLs, on text that is not already inside a link.
	//
	// Most of the doctrine docs write "Full guide: https://..." rather than
	// "[Full guide](https://...)". Without this they render as inert text: the
	// URL is visible but not clickable, which reads as a broken link.
	//
	// The incoming text is ALREADY html-escaped (every caller runs escapeHtml
	// first), so `&` is `&amp;` and `"` is `&quot;` on the way in. Escaping the
	// captured URL a second time would produce `&amp;amp;` and `&amp;quot;`,
	// which corrupts every query string and makes `&quot;` visible in the
	// href. The captured text is therefore emitted as-is; it is already safe,
	// and a raw `"` could not have survived the first pass.
	//
	// Only http/https (and bare www, promoted to https) are matched, so a
	// `javascript:` or `data:` URL can never become an anchor. Trailing
	// sentence punctuation is excluded so "see https://x.com/a." keeps its
	// period outside the link.
	// The TRAILING class excludes `*` and `_` as well as sentence punctuation.
	// Without that, `*Source: https://x/y*` produced href="https://x/y</em>": the
	// italic pass ran afterwards and inserted its own closing tag INSIDE the
	// attribute, so the link pointed nowhere while the visible text looked right.
	// Measured across the eleven doctrine docs, that repair changes one line of
	// 10_organizing_bnwo.md and nothing else.
	//
	// The LEADING class stays as it was. Allowing a `*` there would let a URL
	// preceded by a bold marker match, but the marker is consumed as `pre` and
	// the surviving one renders as a stray asterisk beside the link -- write
	// `[text](url)` when you want a bold link. Pin that in markdown.test.ts.
	s = s.replace(
		/(^|[\s(>])((?:https?:\/\/|www\.)[^\s<>()[\]]+[^\s<>()[\].,;:!?'"*_])/g,
		(m, pre: string, url: string) => {
			const href = url.startsWith('www.') ? `https://${url}` : url;
			return `${pre}<a href="${href}" target="_blank" rel="noopener">${url}</a>`;
		}
	);
	// bold **text**
	s = s.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
	// italic *text* (avoid colliding with bold)
	s = s.replace(/(^|[^*])\*([^*\n]+)\*(?!\*)/g, '$1<em>$2</em>');
	// inline code
	s = s.replace(/`([^`]+)`/g, '<code>$1</code>');
	return s;
}
