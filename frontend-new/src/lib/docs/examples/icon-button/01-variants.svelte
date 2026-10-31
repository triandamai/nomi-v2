<!-- Variants: outline and ghost for tools, send for the composer. Every one has a label. -->
<script lang="ts">
	import { ArrowUp, Mic, Plus } from '@lucide/svelte';
	import IconButton from '$lib/ds/actions/IconButton.svelte';
</script>

<IconButton icon={Plus} label="Attach a file" />
<IconButton icon={Mic} label="Record a voice note" variant="ghost" />
<IconButton icon={ArrowUp} label="Send" variant="send" />
<IconButton icon={ArrowUp} label="Send" variant="send" disabled />
