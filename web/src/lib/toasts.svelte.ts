// Toast store (Svelte 5 runes)
export interface Toast {
	id: number;
	message: string;
	type: 'success' | 'error';
}

let toasts = $state<Toast[]>([]);
let nextId = 1;

export function pushToast(message: string, type: 'success' | 'error' = 'success') {
	const id = nextId++;
	toasts = [...toasts, { id, message, type }];
	setTimeout(() => {
		toasts = toasts.filter((t) => t.id !== id);
	}, 4200);
}

export function getToasts() {
	return toasts;
}

export function dismissToast(id: number) {
	toasts = toasts.filter((t) => t.id !== id);
}
