<!-- A view in a chat bubble (a `{kind: "view"}` block): drawn by its component from
     $lib/views/registry. A view this build doesn't know shows nothing; the message's text stands in. -->
<script lang="ts">
	import { viewComponent } from '$lib/views/registry';
	import type { View } from '$lib/views/types';

	let { block, agent = null, answered = false }: { block: View; agent?: string | null; answered?: boolean } = $props();

	const Component = $derived(viewComponent(block));
</script>

{#if Component}<Component view={block} {agent} {answered} />{/if}
