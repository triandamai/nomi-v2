<script lang="ts">
	import AgentShape from '../../m3/AgentShape.svelte';
	import type { CrewId } from './crew';

	// Twin of frontend-new's Mascot: the crew member's own shape and gradient (agentLook knows the
	// crew by name), with the two-dot face. Decorative next to their name; labelled when alone.
	let { agent, size = 96, label }: { agent: CrewId; size?: 96 | 32 | 20 | 16; label?: string } = $props();
</script>

<AgentShape {agent} {size} {label} face class="ds-mascot" />

<style>
	:global(.ds-mascot) {
		flex-shrink: 0;
	}
</style>
