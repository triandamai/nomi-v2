<!-- Thirteen weeks with a few busy days, under the two numbers. -->
<script lang="ts">
	import PhoneActivity from '$lib/ds/profile/PhoneActivity.svelte';
	import { heatmap } from '$lib/settings/profile';

	const activity = Array.from({ length: 70 }, (_, index) => ({
		date: new Date(Date.UTC(2026, 7, 14 + index)).toISOString().slice(0, 10),
		tokens: ((index * 37) % 11) * 900,
	}));
	const map = heatmap(activity, '2026-10-21', 'en', 13);
</script>

<div class="w-97.5 max-w-full px-4">
	<PhoneActivity
		tiles={[
			{ label: 'Lifetime tokens', value: '1.2M', note: 'since 3 Oct 2026' },
			{ label: 'Current streak', value: '4 days', note: 'best is 9 · 5 to go' },
		]}
		total="412K tokens in the last 13 weeks"
		weeks={map.weeks}
		note="Most active Tuesday"
	/>
</div>
