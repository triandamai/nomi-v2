<!-- People: a search by email in the address, and the list's own action at the end. -->
<script lang="ts">
	import FilterBar from '$lib/ds/admin/FilterBar.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
</script>

<div class="w-full max-w-180">
	<FilterBar label="Search people" placeholder="Search by email" submitLabel="Search" keep={{ size: '50' }}>
		{#snippet actions()}<Button label="Export" variant="ghost" />{/snippet}
	</FilterBar>
</div>
