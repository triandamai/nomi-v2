<!-- A year: a few busy weeks in the autumn. -->
<script lang="ts">
	import ActivityHeatmap from '$lib/ds/profile/ActivityHeatmap.svelte';
	import { heatmap } from '$lib/settings/profile';

	const activity = Array.from({ length: 60 }, (_, index) => ({
		date: new Date(Date.UTC(2026, 7, 1 + index)).toISOString().slice(0, 10),
		tokens: ((index * 37) % 11) * 900,
	}));
	const map = heatmap(activity, '2026-10-21', 'en');
</script>

<div class="w-full max-w-210">
	<ActivityHeatmap label="Token activity" weeks={map.weeks} months={map.months} days={['Mon', 'Wed', 'Fri']} note="Most active Tue · joined 14 Jul" less="Less" more="More" />
</div>
