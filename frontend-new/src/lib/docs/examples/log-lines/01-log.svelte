<!-- The app’s last lines. -->
<script lang="ts">
	import LogLines from '$lib/ds/projects/LogLines.svelte';
</script>

<div class="w-full max-w-170">
	<LogLines label="Live log" lines={['10:54:12 POST /order 201 142ms', '10:53:55 GET /menu 200 38ms', '10:53:58 WARN slow query 412ms', '10:54:01 ERROR db timeout']} />
</div>
