<script lang="ts">
	import type { ComponentProps } from 'svelte';
	import Modal from '../overlays/Modal.svelte';
	import BuildSharePanel from './BuildSharePanel.svelte';

	// Build → Share as a modal: closes with its own close, Done, or Escape.
	type Props = Omit<ComponentProps<typeof BuildSharePanel>, 'titleId' | 'onclose' | 'ondone'> & { open?: boolean; onclose?: () => void };
	let { open = $bindable(false), onclose, ...panel }: Props = $props();

	const id = $props.id();
</script>

<Modal bind:open labelledby="{id}-title" {onclose}>
	<BuildSharePanel {...panel} titleId="{id}-title" onclose={() => (open = false)} ondone={() => (open = false)} />
</Modal>
