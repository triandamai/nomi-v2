<script lang="ts">
	import { Paperclip } from '@lucide/svelte';
	import AttachMenu from './AttachMenu.svelte';
	import IconButton from '../actions/IconButton.svelte';

	// An AttachMenu off the paperclip, for its test and the gallery.
	let { onpick }: { onpick: (source: string) => void } = $props();
</script>

<AttachMenu driveBy="Tara" max="20 MB" {onpick}>
	{#snippet trigger(props)}<IconButton icon={Paperclip} label="Attach" variant="ghost" {...props} />{/snippet}
</AttachMenu>
