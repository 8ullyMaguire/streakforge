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
});
