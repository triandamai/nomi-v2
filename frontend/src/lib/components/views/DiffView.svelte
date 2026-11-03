<!-- A change to one file, line by line (Chat responses → Diff). The hunks are worked out on the
     server from the text before and after. Nothing is applied from here: a crew member's own file
     changes come as file-change blocks. -->
<script lang="ts">
	import Diff from '$lib/components/ds/chat/Diff.svelte';
	import type { DiffView, ViewProps } from '$lib/views/types';

	let { view }: ViewProps<DiffView> = $props();
</script>

<Diff path={view.path} hunks={view.hunks} />
