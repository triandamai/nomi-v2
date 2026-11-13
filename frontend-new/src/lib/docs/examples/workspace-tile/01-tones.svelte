<!-- The three tones, at 40 and 56. -->
<script lang="ts">
	import WorkspaceTile from '$lib/ds/settings/WorkspaceTile.svelte';
</script>

<div class="flex items-center gap-3">
	<WorkspaceTile initial="M" tone="plain" />
	<WorkspaceTile initial="T" tone="teal" />
	<WorkspaceTile initial="K" tone="amber" />
	<WorkspaceTile initial="T" tone="teal" size={56} />
</div>
