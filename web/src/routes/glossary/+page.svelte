<script lang="ts">
	import { TERMS } from '$lib/glossary';
</script>

<svelte:head>
	<title>Glossary — StreakForge</title>
	<meta
		name="description"
		content="WLW, WLD, the Devotion Index, the multiplier and the 30-day decay — every term this site uses, defined."
	/>
</svelte:head>

<div class="container" style="max-width:720px;padding-top:28px;">
	<h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 4px;">GLOSSARY</h1>
	<p style="color:var(--text-dim);font-size:14px;margin:0 0 8px;">
		Every noun this site uses. The scores are made of these words, so a wrong
		idea about one of them is a wrong idea about your own number.
	</p>
	<p style="color:var(--text-dim);font-size:12px;margin:0 0 20px;">
		Each entry names where the rule is enforced — nothing here is decoration.
	</p>

	{#each TERMS as t (t.term)}
		<div class="card" style="margin-bottom:12px;padding:14px 16px;">
			<div
				style="font-family:var(--font-mono);font-size:14px;letter-spacing:0.06em;color:var(--gold);margin-bottom:6px;"
			>
				{t.term}
			</div>
			<div style="font-size:14px;line-height:1.55;">{t.definition}</div>
			<div style="font-size:11px;color:var(--text-dim);margin-top:8px;line-height:1.5;">
				{t.enforced}
			</div>
		</div>
	{/each}

	<p style="color:var(--text-dim);font-size:12px;margin-top:20px;">
		The full rule set, with the reasoning, is in the
		<a href="/doctrine">doctrine</a>. The community numbers are on the
		<a href="/kpi">KPI page</a>.
	</p>
</div>
