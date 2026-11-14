<!-- Requests today, and a number that wasn’t measured. -->
<script lang="ts">
	import MonitorStat from '$lib/ds/projects/MonitorStat.svelte';
</script>

<div class="flex w-full max-w-170 gap-2">
	<MonitorStat label="Requests" value="214" note="today" />
	<MonitorStat label="Errors" value="0" note="0% of requests" />
	<MonitorStat label="P95" notMeasured="Not measured" />
</div>
