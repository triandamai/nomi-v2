<script lang="ts">
	import type { ComponentProps } from 'svelte';
	import AgentReply from './AgentReply.svelte';
	import Prose from './Prose.svelte';

	// For the tests: a reply with its prose, as the chat composes it.
	let { version, dimmed = false }: Pick<ComponentProps<typeof AgentReply>, 'version' | 'dimmed'> = $props();
</script>

<AgentReply agent="dana" name="Dana" meta="14:05 · 9s" {version} {dimmed}>
	<Prose html="<p>Rp 1,8 jt on food in October once office lunches are taken out.</p>" />
</AgentReply>
