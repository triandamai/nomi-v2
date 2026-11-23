<!-- The 23rd: the days so far, today in the accent. -->
<script lang="ts">
	import PhoneDayChart from '$lib/ds/settings/PhoneDayChart.svelte';
	import type { DayBar } from '$lib/ds/settings/DayBars.svelte';

	const HEIGHTS = [6, 16, 2, 24, 44, 18, 8, 2, 30, 60, 36, 14, 52, 82, 24, 10, 2, 38, 66, 48, 22, 16, 4];
	const bars: DayBar[] = Array.from({ length: 30 }, (_, index) => ({
		day: index + 1,
		value: HEIGHTS[index] ?? 0,
		share: (HEIGHTS[index] ?? 0) / 82,
		state: index === 22 ? 'today' : index > 22 ? 'future' : 'past',
	}));
</script>

<div class="w-97.5 max-w-full px-4">
	<PhoneDayChart label="Each day of November" {bars} describe={(bar) => `${bar.day} Nov: ${bar.value}K tokens`} />
</div>
