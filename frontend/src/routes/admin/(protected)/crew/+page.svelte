<!-- Admin → Agents: every built-in crew member, switched on or off for everyone. -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { agentLook } from '$lib/components/m3/shapes';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
</script>

<PageHeader title={m.admin_agent_switches()} lede={m.crewadmin_lede()} />

{#if form?.error}
	<p class="error" role="alert">{form.error}</p>
{/if}

{#if data.agents.length === 0}
	<p class="note">{m.crewadmin_empty()}</p>
{:else}
	<ul class="grid">
		{#each data.agents as agent (agent.agent_type)}
			{@const look = agentLook(agent.agent_type)}
			<li class="card" class:card--off={!agent.enabled} data-agent={agent.agent_type}>
				<div class="card__top">
					<AgentShape shape={look.shape} tone={look.tone} size={52} />
					<div class="card__text">
						<h3 class="card__name">{agent.name}</h3>
						<p class="card__meta">
							<span class="chip">{agent.agent_type}</span>
							{#if agent.locked}
								<span class="chip">{m.crewadmin_always_on()}</span>
							{:else if !agent.enabled}
								<span class="chip chip--off">{m.dyn_off()}</span>
							{/if}
						</p>
					</div>
				</div>
				<p class="card__role">{agent.locked ? `${agent.role} ${m.crewadmin_locked_note()}` : agent.role}</p>
				{#if !agent.locked}
					<form method="POST" action="?/toggle" use:enhance class="card__actions">
						<input type="hidden" name="agent_type" value={agent.agent_type} />
						<input type="hidden" name="enabled" value={agent.enabled ? 'false' : 'true'} />
						<Button type="submit" variant={agent.enabled ? 'text' : 'tonal'} size="xs">{agent.enabled ? m.dyn_turn_off() : m.dyn_turn_on()}</Button>
					</form>
				{/if}
			</li>
		{/each}
	</ul>
	<p class="note">{m.crewadmin_custom()} <a href="/admin/dynamic-agents">{m.admin_custom_agents()}</a></p>
{/if}

<style>
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(min(100%, 320px), 1fr));
		gap: 12px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.card {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 16px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.card--off {
		background: var(--md-sys-color-surface-container);
	}
	.card--off :global(.agent-shape) {
		filter: grayscale(0.8);
		opacity: 0.6;
	}
	.card__top {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.card__text {
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
	}
	.card__name {
		margin: 0;
		font-size: 1rem;
		font-weight: 600;
	}
	.card__meta {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		margin: 0;
	}
	.card__role {
		flex: 1;
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		line-height: 1.45;
	}
	.card__actions {
		display: flex;
		justify-content: flex-end;
	}
	.chip {
		padding: 2px 8px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
	}
	.chip--off {
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}
	.note {
		margin: 16px 0 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.note a {
		color: var(--md-sys-color-primary);
	}
	.error {
		margin: 0 0 12px;
		color: var(--md-sys-color-error);
		font-size: 0.875rem;
	}
</style>
