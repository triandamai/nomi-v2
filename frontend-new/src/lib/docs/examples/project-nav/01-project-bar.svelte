<!-- Project bar: the project and whether it's live, the publish pill, and the chat or preview switch under it. -->
<script lang="ts">
	import ProjectNav from '$lib/ds/mobile/ProjectNav.svelte';
	import WorkspaceSwitch from '$lib/ds/mobile/WorkspaceSwitch.svelte';
</script>

<div class="w-97.5 max-w-full">
	<ProjectNav project="warung-kopi-menu" status="live · deploy #2" publish="idle" onback={() => {}} onmenu={() => {}} />
	<WorkspaceSwitch />
</div>
