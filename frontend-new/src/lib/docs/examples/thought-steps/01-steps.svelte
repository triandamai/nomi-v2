<!-- Steps: the real tool, size and time of each, the current one running. -->
<script lang="ts">
	import ThoughtSteps from '$lib/ds/chat/ThoughtSteps.svelte';
</script>

<div class="w-full max-w-170">
	<ThoughtSteps
		label="Worked through 4 steps"
		meta="· 12s"
		steps={[
			{ label: 'Read October transactions', meta: 'dana.records · 142 rows · 1.1s', state: 'done' },
			{ label: 'Grouped by category and hour', meta: 'dana.categorize · 0.4s', state: 'done' },
			{ label: 'Compared with September', meta: 'dana.compare · 0.3s', state: 'done' },
			{ label: 'Drafting the answer', meta: 'nomi · now', state: 'running' },
		]}
	/>
</div>
