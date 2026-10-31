<!-- On an icon button: hover it, or Tab to it. The tip uses the button's own name. -->
<script lang="ts">
	import { Plus } from '@lucide/svelte';
	import IconButton from '$lib/ds/actions/IconButton.svelte';
	import Tooltip from '$lib/ds/overlays/Tooltip.svelte';
</script>

<Tooltip label="New chat" shortcut="⌘K">
	{#snippet children(props)}<IconButton icon={Plus} label="New chat" {...props} />{/snippet}
</Tooltip>
