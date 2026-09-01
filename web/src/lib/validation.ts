export function isValidUsername(u: string): boolean {
	return (
		u.length >= 2 &&
		u.length <= 30 &&
		/^[a-zA-Z0-9_]+$/.test(u)
	);
}
