<!-- A live project: its tile, what it is, where it lives, and when it last changed. -->
<script lang="ts">
	import ProjectRow from '$lib/ds/lists/ProjectRow.svelte';
</script>

<div class="w-full max-w-200">
	<ProjectRow
		href="#salah-daily-tracker"
		name="salah-daily-tracker"
		description="Daily salah tracker with countdown and streaks"
		tone="nomi"
		status={{ kind: 'live', label: 'Live' }}
		url="salah-daily-tracker-0ab8f8.trian.space"
		updated="2d"
	/>
</div>
