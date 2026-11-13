<!-- Default: the selected project, what's in it, and its chats. -->
<script lang="ts">
	import ProjectDetailPanel from '$lib/ds/projects/ProjectDetailPanel.svelte';
</script>

<ProjectDetailPanel
	name="salah-daily-tracker"
	tone="nomi"
	live="live · deploy #14 · 2d ago"
	workspaceHref="#workspace"
	url="salah-daily-tracker-0ab8f8.trian.space"
	parts={{ code: '23 files · main@a3f9c21', url: '…0ab8f8.trian.space', data: 'localStorage only · no server', deploys: '14 · last built in 41s', members: 'Trian (owner), you' }}
	onexport={() => {}}
	chats={[
		{ title: 'Build a daily salah tracker', age: '2d', href: '#c1' },
		{ title: 'Add Hijri offset setting', age: '2d', href: '#c2' },
	]}
	onask={() => {}}
	onclose={() => {}}
>
	{#snippet preview()}
		<div class="h-full bg-surface-2"></div>
	{/snippet}
</ProjectDetailPanel>
