<!-- Sizes: 32 in the feed, 26 in Coming up. -->
<script lang="ts">
	import SourceTile from '$lib/ds/inbox/SourceTile.svelte';
</script>

<div class="flex items-center gap-3">
	<SourceTile agent="kala" />
	<SourceTile agent="koda" />
	<SourceTile agent="dana" />
	<SourceTile agent="kala" size={26} />
	<SourceTile agent="dana" size={26} />
</div>
