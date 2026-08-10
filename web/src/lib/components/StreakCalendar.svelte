<script lang="ts">
	// 30-day streak calendar heatmap. `days` is [{date: 'YYYY-MM-DD', count}] ending today.
	let { days }: { days: { date: string; count: number }[] } = $props();

	function intensity(count: number): string {
		if (count <= 0) return 'rgba(255,255,255,0.06)';
		if (count === 1) return 'rgba(227,28,35,0.35)';
		if (count === 2) return 'rgba(227,28,35,0.6)';
		if (count === 3) return 'rgba(227,28,35,0.8)';
		return 'rgba(227,28,35,1)';
	}
	function fmt(d: string): string {
		const [y, m, day] = d.split('-').map(Number);
		const dt = new Date(Date.UTC(y, m - 1, day));
		return dt.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
	}
</script>

<div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(18px,1fr));gap:6px;">
	{#each days as d (d.date)}
		<div
			title={`${fmt(d.date)} — ${d.count} log${d.count === 1 ? '' : 's'}`}
			style="aspect-ratio:1;border-radius:4px;background:{intensity(d.count)};"
		></div>
	{/each}
</div>
