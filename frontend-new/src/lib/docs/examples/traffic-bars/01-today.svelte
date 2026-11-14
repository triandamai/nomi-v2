<!-- A morning rush. -->
<script lang="ts">
	import TrafficBars from '$lib/ds/projects/TrafficBars.svelte';

	const counts = [0, 0, 0, 0, 0, 0, 0, 0, 6, 120, 126, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
	const bars = counts.map((requests, hour) => ({ at: `${String(hour).padStart(2, '0')}:00`, share: requests / 126, requests }));
</script>

<div class="w-full max-w-170">
	<TrafficBars label="Requests per hour, today" {bars} axis={['00', '06', '12', '18', '24']} />
</div>
