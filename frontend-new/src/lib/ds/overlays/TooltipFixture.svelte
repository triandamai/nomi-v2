<script lang="ts">
	import { Plus } from '@lucide/svelte';
	import Tooltip from './Tooltip.svelte';
	import IconButton from '../actions/IconButton.svelte';
</script>

<Tooltip label="New chat" shortcut="⌘K">
	{#snippet children(props)}<IconButton icon={Plus} label="New chat" {...props} />{/snippet}
</Tooltip>
