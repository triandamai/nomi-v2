<!-- A built-in member’s mascot, and a custom member’s initial. -->
<script lang="ts">
	import CrewMark from '$lib/ds/settings/CrewMark.svelte';
</script>

<div class="flex items-center gap-3">
	<CrewMark agent="dana" />
	<CrewMark initial="B" />
	<CrewMark agent="koda" size={32} />
	<CrewMark initial="B" size={32} />
</div>
