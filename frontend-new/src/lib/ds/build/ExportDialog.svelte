<script lang="ts">
	import type { ComponentProps } from 'svelte';
	import Modal from '../overlays/Modal.svelte';
	import ExportPanel from './ExportPanel.svelte';

	// Build → Export as a modal: closes with its own close or Escape.
	type Props = Omit<ComponentProps<typeof ExportPanel>, 'titleId' | 'onclose'> & { open?: boolean; onclose?: () => void };
	let { open = $bindable(false), onclose, ...panel }: Props = $props();

	const id = $props.id();
</script>

<Modal bind:open labelledby="{id}-title" {onclose}>
	<ExportPanel {...panel} titleId="{id}-title" onclose={() => (open = false)} />
</Modal>
