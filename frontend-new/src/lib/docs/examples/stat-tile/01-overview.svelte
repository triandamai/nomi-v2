<!-- The overview's first row: live while agents are running. -->
<script lang="ts">
	import StatTile from '$lib/ds/admin/StatTile.svelte';
</script>

<div class="grid w-full max-w-180 grid-cols-2 gap-3 sm:grid-cols-4">
	<StatTile label="Running now" value="3" note="agents working" live />
	<StatTile label="People" value="1.2K" exact="1,234" note="with an account" />
	<StatTile label="Tokens today" value="48.2K" exact="48,210" note="across every agent" />
	<StatTile label="Tokens all time" value="12.6M" exact="12,604,331" note="since launch" />
</div>
