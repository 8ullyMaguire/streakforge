<script lang="ts">
	import { page } from '$app/state';
</script>

<div class="container" style="text-align:center;padding:80px 20px;">
	<h1 style="font-size:64px;margin:0;">{page.status}</h1>
	<p style="color:var(--text-dim);">Something went wrong on our forge.</p>
	<a href="/" class="btn btn-red" style="margin-top:16px;">BACK TO HALL</a>
</div>
