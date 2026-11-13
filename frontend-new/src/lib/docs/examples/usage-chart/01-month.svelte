<!-- A month of chat tokens: one ink, a zero baseline, an honest round scale. -->
<script lang="ts">
	import UsageChart from '$lib/ds/admin/UsageChart.svelte';

	const values = [12, 30, 18, 0, 44, 52, 38, 9, 61, 70, 33, 21, 0, 15, 48, 57, 66, 41, 29, 35, 72, 80, 64, 22, 18, 47, 55, 39, 26, 31];
	const bars = values.map((value, index) => ({
		key: String(index + 1),
		value: value * 1000,
		tick: index === 0 || (index + 1) % 7 === 0 ? String(index + 1) : undefined,
		tooltip: [`${index + 1} Nov`, `${value * 1000} chat tokens`],
	}));
</script>

<div class="w-full max-w-180">
	<UsageChart title="Chat tokens per day" {bars} max={100_000} axis={['100K', '50K', '0']} />
</div>
