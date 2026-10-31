<script lang="ts">
	import { Link2 } from '@lucide/svelte';
	import WorkspaceSheet from './WorkspaceSheet.svelte';

	// A WorkspaceSheet with placeholder content, for its test and the gallery.
	let { onclose = () => {} }: { onclose?: () => void } = $props();
</script>

<div class="h-105">
	<WorkspaceSheet icon={Link2} title="Domain" meta="warung-kopi-menu" onexpand={() => {}} {onclose}>
		<p class="text-ui text-text-3">Sheet content</p>
	</WorkspaceSheet>
</div>
