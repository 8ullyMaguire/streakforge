import { describe, it, expect } from 'vitest';
import { hex, computeChallengeProof } from './challenge';

describe('hex', () => {
	it('encodes bytes as lowercase hex', () => {
		// "abc" in UTF-8
		expect(hex(new TextEncoder().encode('abc').buffer)).toBe('616263');
	});
	it('pads single bytes with a leading zero', () => {
		expect(hex(Uint8Array.from([0x0f, 0x01]).buffer)).toBe('0f01');
	});
});

describe('computeChallengeProof', () => {
	it('returns the first 16 hex chars of sha256(nonce + username + password)', async () => {
		const proof = await computeChallengeProof('deadbeef', 'alice', 'password123');
		expect(proof).toBe('6309588508e5f3b4');
		expect(proof).toMatch(/^[0-9a-f]{16}$/);
	});

	it('depends on the nonce', async () => {
		const a = await computeChallengeProof('nonce1', 'alice', 'password123');
		const b = await computeChallengeProof('nonce2', 'alice', 'password123');
		expect(a).not.toBe(b);
	});

	it('depends on the password (not just username)', async () => {
		const a = await computeChallengeProof('nonce1', 'alice', 'password123');
		const b = await computeChallengeProof('nonce1', 'alice', 'differentpass');
		expect(a).not.toBe(b);
	});
});
