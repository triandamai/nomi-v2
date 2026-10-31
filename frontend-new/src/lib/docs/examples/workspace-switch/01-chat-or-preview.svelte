<!-- Chat or preview: one screen, switched here. -->
<script lang="ts">
	import WorkspaceSwitch from '$lib/ds/mobile/WorkspaceSwitch.svelte';

	let view = $state<'chat' | 'preview'>('chat');
</script>

<div class="w-97.5 max-w-full">
	<WorkspaceSwitch bind:value={view} />
</div>
