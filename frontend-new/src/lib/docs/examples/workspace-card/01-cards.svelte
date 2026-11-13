<!-- The workspace projects come from, and another. -->
<script lang="ts">
	import WorkspaceCard from '$lib/ds/settings/WorkspaceCard.svelte';

	const labels = { settings: 'Settings', currentTag: 'Current', currentNote: 'Projects come from here', use: 'Use for projects' };
</script>

<div class="grid w-full max-w-200 gap-4 md:grid-cols-2">
	<WorkspaceCard name="Trian’s space" initial="T" tone="teal" meta="Admin · 3 members" stats={[{ value: 4, label: 'projects' }, { value: 3, label: 'members' }]} current href="#" {...labels} />
	<WorkspaceCard name="mario" initial="M" tone="plain" meta="Owner · only you" stats={[{ value: 1, label: 'project' }, { value: 1, label: 'member' }]} current={false} href="#" {...labels} />
</div>
