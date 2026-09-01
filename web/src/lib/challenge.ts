// Proof-of-work challenge for the auth form.
//
// The backend issues a random nonce via GET /api/auth/nonce. The client must
// return the first 16 hex chars of sha256(nonce + username + password).
// Combined with a honeypot field and a minimum time-on-page, this dissuades
// naive bots from mass-registering accounts.

const encoder = new TextEncoder();

export function hex(buffer: ArrayBuffer): string {
	return Array.from(new Uint8Array(buffer))
		.map((b) => b.toString(16).padStart(2, '0'))
		.join('');
}

export function computeChallengeProof(
	nonce: string,
	username: string,
	password: string
): Promise<string> {
	return crypto.subtle
		.digest('SHA-256', encoder.encode(`${nonce}${username}${password}`))
		.then((buf) => hex(buf).slice(0, 16));
}
