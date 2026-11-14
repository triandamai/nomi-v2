<!-- A page inside another: the way back, and its id in mono. -->
<script lang="ts">
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
</script>

<div class="w-full max-w-180">
	<PageHeader title="Pro" meta="pro" back={{ href: '#plans', label: 'Plans' }} />
</div>
