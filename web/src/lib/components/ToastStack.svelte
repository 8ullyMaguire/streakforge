<script lang="ts">
	import { getToasts, dismissToast } from '$lib/toasts.svelte';
	import { X } from 'lucide-svelte';
	const toasts = getToasts();
</script>

<div class="toast-stack">
	{#each toasts as t (t.id)}
		<div class="toast" class:toast-error={t.type === 'error'}>
			<span>{t.message}</span>
			<button
				style="background:none;border:none;color:inherit;margin-left:8px;cursor:pointer;"
				onclick={() => dismissToast(t.id)}
			>
				<X size={14} />
			</button>
		</div>
	{/each}
</div>
