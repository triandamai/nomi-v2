<script lang="ts">
	import Card from '$lib/components/m3/Card.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Running agents</h1>
<p class="md-body-large mt-2" style="color: var(--md-sys-color-on-surface-variant)">
	Every currently active agent session, grouped by user.
</p>

<div class="mt-6 space-y-4">
	{#if data.agents.users.length === 0}
		<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">
			No agents are currently running.
		</p>
	{:else}
		{#each data.agents.users as group (group.user_id)}
			<Card variant="outlined" class="p-4">
				<p class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{group.label}</p>
				<div class="mt-2 space-y-1">
					{#each group.agents as agent (agent.agent_session_id)}
						<div class="md-body-medium flex justify-between" style="color: var(--md-sys-color-on-surface-variant)">
							<span>{agent.agent_type} · {agent.channel}</span>
							<span>started {new Date(agent.started_at).toLocaleString()}</span>
						</div>
					{/each}
				</div>
			</Card>
		{/each}
	{/if}
</div>
