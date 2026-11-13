<!-- Drawn open: the panel as the sheet shows it. In a page, use SortButton. -->
<script lang="ts">
	import SortMenu from '$lib/ds/projects/SortMenu.svelte';

	let value = $state<'updated' | 'name' | 'status' | 'created'>('updated');
</script>

<SortMenu {value} onchoose={(sort) => (value = sort)} />
