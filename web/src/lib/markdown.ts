// Tiny markdown renderer — enough for the manifesto docs.
// Handles: headings, bold, italic, links, inline code, code blocks, lists,
// blockquotes, paragraphs, hr. No HTML passthrough (escape everything first).

export function renderMarkdown(src: string): string {
	const lines = src.replace(/\r\n/g, '\n').split('\n');
	const out: string[] = [];
	let inCode = false;
	let inList = false;
	let codeBuf: string[] = [];

	function closeList() {
		if (inList) {
			out.push('</ul>');
			inList = false;
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
			continue;
		}

		// paragraph
		closeList();
		out.push(`<p>${inline(escapeHtml(trimmed))}</p>`);
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
			return `<a href="${escapeHtml(safe)}" target="_blank" rel="noopener">${escapeHtml(text)}</a>`;
		}
	);
	// Bare URLs, on text that is not already inside a link.
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
	s = s.replace(
		/(^|[\s(>])((?:https?:\/\/|www\.)[^\s<>()[\]]+[^\s<>()[\].,;:!?'"])/g,
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
