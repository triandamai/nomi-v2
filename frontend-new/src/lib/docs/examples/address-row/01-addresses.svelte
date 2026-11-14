<!-- The live address, and the one it moves to on the next publish. -->
<script lang="ts">
	import AddressRow from '$lib/ds/projects/AddressRow.svelte';
</script>

<div class="flex w-full max-w-170 flex-col gap-2">
	<AddressRow address="warung-kopi-menu-3f9a12.trian.space" href="#" state={{ label: 'Live', tone: 'live' }} />
	<AddressRow address="warung-kopi.trian.space" state={{ label: 'on the next publish', tone: 'waiting' }} strong />
</div>
