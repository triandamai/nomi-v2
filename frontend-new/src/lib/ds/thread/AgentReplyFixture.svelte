<script lang="ts">
	import type { ComponentProps } from 'svelte';
	import AgentReply from './AgentReply.svelte';
	import Prose from './Prose.svelte';

	// For the tests: a reply with its prose, as the chat composes it.
	let { version, dimmed = false, phone = false }: Pick<ComponentProps<typeof AgentReply>, 'version' | 'dimmed' | 'phone'> = $props();
</script>

<AgentReply agent="dana" name="Dana" meta="14:05 · 9s" {version} {dimmed} {phone}>
	<Prose html="<p>Rp 1,8 jt on food in October once office lunches are taken out.</p>" />
</AgentReply>
