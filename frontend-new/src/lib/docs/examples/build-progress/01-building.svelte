<!-- Building: the real step, which step of how many, and that the app can be closed. -->
<script lang="ts">
	import BuildProgress from '$lib/ds/mobile/BuildProgress.svelte';
</script>

<div class="w-89.5 max-w-full">
	<BuildProgress project="warung-kopi-menu" elapsed="01:12" step="writing src/routes/menu/+page.svelte" current={2} total={6} label="menu page" eta="~3 min left" onstop={() => {}} />
</div>
