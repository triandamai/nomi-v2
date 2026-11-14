<!-- Two tables, orders shown. -->
<script lang="ts">
	import TableChip from '$lib/ds/projects/TableChip.svelte';
</script>

<div class="flex gap-2">
	<TableChip name="orders" rows="7 rows" selected />
	<TableChip name="menu_items" rows="10 rows" />
</div>
