<!-- October, on the 21st. -->
<script lang="ts">
	import DayBars, { type DayBar } from '$lib/ds/settings/DayBars.svelte';

	const bars: DayBar[] = Array.from({ length: 31 }, (_, index) => {
		const day = index + 1;
		const value = day > 21 ? 0 : ((day * 7) % 5) * 800;
		return { day, value, share: value / 3200, state: day === 21 ? 'today' : day > 21 ? 'future' : value ? 'past' : 'empty' };
	});
</script>

<div class="w-full max-w-141">
	<DayBars label="Tokens each day" {bars} scale={['3.2K', '1.6K', '0']} today="today" describe={(bar) => `${bar.day} Oct: ${bar.value} tokens`} />
</div>
