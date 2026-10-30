<script lang="ts">
	import type { CrewId } from '../agents/crew';

	// Chat responses → Mention / Crew: "@Dana" in that member's own tint, the only place in a
	// message the crew colours appear.
	let { agent, name }: { agent: CrewId; name: string } = $props();
</script>

<span class="inline-flex items-center rounded-xs px-1.5 py-px align-top text-ui font-medium" style="background: var(--crew-{agent}-bg); color: var(--crew-{agent}-ink)">@{name}</span>
