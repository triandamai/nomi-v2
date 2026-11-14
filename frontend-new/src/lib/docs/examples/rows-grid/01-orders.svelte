<!-- The newest pre-orders. -->
<script lang="ts">
	import RowsGrid from '$lib/ds/projects/RowsGrid.svelte';
</script>

<div class="w-full max-w-170">
	<RowsGrid
		label="orders"
		columns={['id', 'name', 'items', 'pickup', 'total', 'status']}
		rows={[
			['0007', 'Rina', '2 kopi susu, 1 roti bakar', '07:30', '48000', 'new'],
			['0006', 'Pak Hadi', '1 kopi tubruk', '07:15', '10000', 'new'],
			['0005', 'Dewi', '2 es kopi', '07:00', '40000', 'picked up'],
		]}
	/>
</div>
