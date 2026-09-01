<script lang="ts">
	// wlw-style scrolling marquee of recent usernames
	let { items }: { items: { username: string; time: string }[] } = $props();
	let groups = $derived([items, items]);
</script>

<div class="marquee" aria-hidden="true">
	<div class="marquee-track">
		{#each groups as group, gi (gi)}
			<div class="marquee-group">
				{#each group as item, i (gi + '-' + i)}
					<span class="dot"></span>
					<span>{item.username}</span>
					<span style="color:var(--text-dim);">{item.time}</span>
				{/each}
			</div>
		{/each}
	</div>
</div>
