<!-- Searching: the focus stroke, the clear, and Cancel in Sort's place. -->
<script lang="ts">
	import PhoneProjectSearch from '$lib/ds/projects/PhoneProjectSearch.svelte';

	let value = $state('warung');
</script>

<div class="w-97.5 max-w-full"><PhoneProjectSearch placeholder="Search projects" sortLabel="Sort" bind:value onsort={() => {}} /></div>
