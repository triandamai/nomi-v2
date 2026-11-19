<!-- Monitor's tiles: one measured, one not. -->
<script lang="ts">
	import PhoneTile from '$lib/ds/projects/PhoneTile.svelte';
</script>

<div class="flex w-89.5 max-w-full gap-2">
	<PhoneTile label="Requests" value="128" note="today" notMeasured="Not measured" />
	<PhoneTile label="Errors" value={null} notMeasured="Not measured" />
</div>
