<!-- Numbers: the current streak stands out. -->
<script lang="ts">
	import StatStrip from '$lib/ds/profile/StatStrip.svelte';
</script>

<div class="w-full max-w-210">
	<StatStrip
		tiles={[
			{ value: '186.4K', label: 'Lifetime tokens', note: 'since 14 Jul' },
			{ value: '14.2K', label: 'Peak day', note: '2 Aug' },
			{ value: '4m 12s', label: 'Longest task', note: 'salah-daily-tracker' },
			{ value: '9 days', label: 'Longest streak', note: '1–9 Aug' },
			{ value: '3 days', label: 'Current streak', note: 'best is 9 — 6 to go', current: true },
		]}
	/>
</div>
