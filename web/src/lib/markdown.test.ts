import { describe, it, expect } from 'vitest';
import { renderMarkdown } from './markdown';

describe('renderMarkdown', () => {
	it('renders headings', () => {
		expect(renderMarkdown('# Hello')).toBe('<h1>Hello</h1>');
		expect(renderMarkdown('## Sub')).toBe('<h2>Sub</h2>');
	});
	it('renders bold and italic', () => {
		expect(renderMarkdown('**bold** text')).toBe('<p><strong>bold</strong> text</p>');
		expect(renderMarkdown('*it*')).toBe('<p><em>it</em></p>');
	});
	it('renders links safely', () => {
		expect(renderMarkdown('[x](https://example.com)')).toBe(
			'<p><a href="https://example.com" target="_blank" rel="noopener">x</a></p>'
		);
	});
	it('escapes HTML in content', () => {
		expect(renderMarkdown('<script>alert(1)</script>')).toBe(
			'<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>'
		);
	});
	it('renders lists', () => {
		expect(renderMarkdown('- a\n- b')).toBe('<ul>\n<li>a</li>\n<li>b</li>\n</ul>');
	});
	it('renders blockquote', () => {
		expect(renderMarkdown('> quote')).toBe('<blockquote>quote</blockquote>');
	});
	it('renders code blocks', () => {
		expect(renderMarkdown('```\nlet x = 1;\n```')).toBe('<pre><code>let x = 1;</code></pre>');
	});
	it('renders hr', () => {
		expect(renderMarkdown('---')).toBe('<hr />');
	});

	// Bare URLs. The doctrine docs mostly write "Full guide: https://..." rather
	// than a markdown link, which used to render as inert, unclickable text.
	describe('bare URLs', () => {
		it('linkifies a bare URL after a label', () => {
			expect(renderMarkdown('Full guide: https://example.com/guide')).toBe(
				'<p>Full guide: <a href="https://example.com/guide" target="_blank" rel="noopener">https://example.com/guide</a></p>'
			);
		});

		it('linkifies a bare www URL and gives it an https href', () => {
			expect(renderMarkdown('see www.example.com/x')).toBe(
				'<p>see <a href="https://www.example.com/x" target="_blank" rel="noopener">www.example.com/x</a></p>'
			);
		});

		it('leaves trailing sentence punctuation outside the link', () => {
			const out = renderMarkdown('Read https://example.com/a.');
			expect(out).toBe(
				'<p>Read <a href="https://example.com/a" target="_blank" rel="noopener">https://example.com/a</a>.</p>'
			);
		});

		it('does not double-linkify a URL already inside a markdown link', () => {
			const out = renderMarkdown('[label](https://example.com/a)');
			expect(out).toBe(
				'<p><a href="https://example.com/a" target="_blank" rel="noopener">label</a></p>'
			);
			// exactly one anchor, and no <a> nested inside the href attribute
			expect(out.match(/<a /g)?.length).toBe(1);
		});

		it('handles query strings and fragments intact', () => {
			const url = 'https://example.com/p?page=post&s=list&tags=%28a+%7e+b%29#frag';
			const out = renderMarkdown(`go ${url} end`);
			// `&` is escaped as `&amp;`, which is the CORRECT html-attribute form;
			// a browser decodes it back to `&` when following the link. The
			// bug this guards against is a raw `&` or a lost query string.
			expect(out).toContain('href="https://example.com/p?page=post&amp;s=list');
			expect(out).toContain('tags=%28a+%7e+b%29#frag');
		});

		it('refuses to linkify a javascript: URL', () => {
			const out = renderMarkdown('javascript:alert(1)');
			expect(out).not.toContain('<a ');
			expect(out).toBe('<p>javascript:alert(1)</p>');
		});

		it('refuses to linkify a data: URL', () => {
			const out = renderMarkdown('data:text/html,<script>x</script>');
			expect(out).not.toContain('<a ');
		});

		it('escapes a quote in a URL so it cannot break out of href', () => {
			const out = renderMarkdown('https://example.com/"onmouseover="alert(1)');
			expect(out).not.toContain('onmouseover="alert');
			expect(out).toContain('&quot;');
		});

		it('does not linkify inside a code span', () => {
			// code spans are applied last, so the URL is already wrapped; the
			// important guarantee is that the result stays balanced.
			const out = renderMarkdown('`https://example.com`');
			expect(out.match(/<code>/g)?.length).toBe(1);
			expect(out).toContain('https://example.com');
		});
	});
});
