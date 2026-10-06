<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Checkbox from '$lib/components/m3/Checkbox.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import ShapePicker from '$lib/components/m3/ShapePicker.svelte';
	import {
		isGradientTone,
		isShapeMotion,
		isShapeName,
		type GradientTone,
		type ShapeMotion,
		type ShapeName,
	} from '$lib/components/m3/shapes';
	import TextField from '$lib/components/m3/TextField.svelte';
	import type { ActionData, PageData } from './$types';
	import type { DynamicAgent } from '$lib/types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	// Grouped by source agent, matching the design spec's admin tool-picker requirement.
	const TOOL_GROUPS: { label: string; tools: { name: string; description: string }[] }[] = [
		{
			label: 'Money',
			tools: [
				{ name: 'list_transactions', description: m.tooldesc_list_transactions() },
				{ name: 'summarize_budget', description: m.tooldesc_summarize_budget() },
				{ name: 'log_transaction', description: m.tooldesc_log_transaction() },
				{ name: 'set_budget', description: m.tooldesc_set_budget() },
				{ name: 'list_budgets', description: m.tooldesc_list_budgets() },
			],
		},
		{ label: 'Planning', tools: [{ name: 'create_project', description: m.tooldesc_create_project() }] },
		{
			label: 'Personality',
			tools: [
				{ name: 'set_personality', description: m.tooldesc_set_personality() },
				{ name: 'list_personality_versions', description: m.tooldesc_list_personality_versions() },
				{ name: 'rollback_personality', description: m.tooldesc_rollback_personality() },
			],
		},
		{ label: 'Supervisor', tools: [{ name: 'list_recent_agent_activity', description: m.tooldesc_list_recent_agent_activity() }] },
		{
			label: 'Coding',
			tools: [
				{ name: 'write_file', description: m.tooldesc_write_file() },
				{ name: 'read_file', description: m.tooldesc_read_file() },
				{ name: 'list_files', description: m.tooldesc_list_files() },
				{ name: 'delete_file', description: m.tooldesc_delete_file() },
			],
		},
	];

	let sheetOpen = $state(false);
	let editingId = $state<string | null>(null);
	let name = $state('');
	let systemPrompt = $state('');
	let intentLabel = $state('');
	let intentDescription = $state('');
	let grantedTools = $state<string[]>([]);
	let supportsTodos = $state(false);
	let supportsPlans = $state(false);
	let canDelegate = $state(false);
	let shape = $state<ShapeName>('cookie9');
	let tone = $state<GradientTone>('glow');
	let motion = $state<ShapeMotion>('spin');

	function openCreate() {
		editingId = null;
		name = '';
		systemPrompt = '';
		intentLabel = '';
		intentDescription = '';
		grantedTools = [];
		supportsTodos = false;
		supportsPlans = false;
		canDelegate = false;
		shape = 'cookie9';
		tone = 'glow';
		motion = 'spin';
		sheetOpen = true;
	}

	function openEdit(agent: DynamicAgent) {
		editingId = agent.id;
		name = agent.name;
		systemPrompt = agent.system_prompt;
		intentLabel = agent.intent_label;
		intentDescription = agent.intent_description;
		grantedTools = [...agent.granted_tools];
		supportsTodos = agent.supports_todos;
		supportsPlans = agent.supports_plans;
		canDelegate = agent.can_delegate;
		shape = isShapeName(agent.shape) ? agent.shape : 'cookie9';
		tone = isGradientTone(agent.tone) ? agent.tone : 'glow';
		motion = isShapeMotion(agent.motion) ? agent.motion : 'spin';
		sheetOpen = true;
	}

	function toggleTool(toolName: string, checked: boolean) {
		grantedTools = checked ? [...grantedTools, toolName] : grantedTools.filter((t) => t !== toolName);
	}
</script>

<PageHeader
	title={m.admin_custom_agents()}
	lede={m.dyn_lede()}
>
	{#snippet actions()}
		<Button type="button" variant="filled" onclick={openCreate}><IconPlus size={18} /> {m.dyn_add()}</Button>
	{/snippet}
</PageHeader>

{#if data.agents.length === 0}
	<div class="empty">
		<AgentShape shape="flower5" tone="dusk" size={64} />
		<p class="empty__text"><strong>{m.dyn_empty_title()}</strong> {m.dyn_empty()}</p>
	</div>
{:else}
	<ul class="grid">
		{#each data.agents as agent (agent.id)}
			<li class="card" class:card--off={!agent.is_active}>
				<div class="card__top">
					<AgentShape
						shape={isShapeName(agent.shape) ? agent.shape : 'cookie9'}
						tone={isGradientTone(agent.tone) ? agent.tone : 'glow'}
						size={52}
					/>
					<div class="card__text">
						<h3 class="card__name">{agent.name}</h3>
						<p class="card__meta">
							<span class="chip">{agent.intent_label}</span>
							<span>{agent.granted_tools.length === 1 ? m.dyn_tool_one() : m.dyn_tool_many({ count: agent.granted_tools.length })}</span>
							{#if !agent.is_active}<span class="chip chip--off">{m.dyn_off()}</span>{/if}
						</p>
					</div>
				</div>
				<p class="card__intent">{agent.intent_description}</p>
				<div class="card__actions">
					<Button type="button" variant="tonal" size="xs" onclick={() => openEdit(agent)}>{m.dyn_edit()}</Button>
					<form method="POST" action="?/toggleActive" use:enhance>
						<input type="hidden" name="id" value={agent.id} />
						<Button type="submit" variant="text" size="xs">{agent.is_active ? m.dyn_turn_off() : m.dyn_turn_on()}</Button>
					</form>
				</div>
			</li>
		{/each}
	</ul>
{/if}

<BottomSheet bind:open={sheetOpen}>
	<h2 class="sheet-title">{editingId ? m.dyn_edit_named({ name: name || m.dyn_agent() }) : m.dyn_add()}</h2>

	{#if form?.error}
		<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{form.error}</p>
	{/if}

	<form
		method="POST"
		action={editingId ? '?/update' : '?/create'}
		use:enhance={() => {
			return async ({ update }) => {
				await update({ reset: false });
				sheetOpen = false;
			};
		}}
		class="mt-4 flex flex-col gap-3"
	>
		{#if editingId}
			<input type="hidden" name="id" value={editingId} />
		{/if}
		<TextField id="name" name="name" label={m.dyn_name()} bind:value={name} required />
		<ShapePicker bind:shape bind:tone bind:motion {name} />
		<TextField id="intent_label" name="intent_label" label={m.dyn_intent_label()} bind:value={intentLabel} required />
		<TextField
			id="intent_description"
			name="intent_description"
			label={m.dyn_intent_description()}
			bind:value={intentDescription}
			required
		/>
		<label class="prompt">
			<span class="prompt__label">{m.dyn_system_prompt()}</span>
			<textarea name="system_prompt" bind:value={systemPrompt} required rows="6" class="prompt__field"></textarea>
		</label>

		<fieldset class="tools">
			<legend class="section-label">{m.dyn_tools()}</legend>
			{#each TOOL_GROUPS as group (group.label)}
				<div class="tools__group">
					<p class="tools__owner"><AgentShape agent={group.label} size={18} /> {group.label}</p>
					{#each group.tools as tool (tool.name)}
						<Checkbox
							name="granted_tools"
							value={tool.name}
							checked={grantedTools.includes(tool.name)}
							onchange={(e) => toggleTool(tool.name, (e.currentTarget as HTMLInputElement).checked)}
							label={tool.description}
						/>
					{/each}
				</div>
			{/each}
		</fieldset>

		<fieldset class="tools">
			<legend class="section-label">{m.dyn_abilities()}</legend>
			<Checkbox bind:checked={supportsTodos} label={m.dyn_todos()} />
			<Checkbox bind:checked={supportsPlans} label={m.dyn_plans()} />
			<Checkbox bind:checked={canDelegate} label={m.dyn_delegate()} />
		</fieldset>
		<input type="hidden" name="supports_todos" value={supportsTodos} />
		<input type="hidden" name="supports_plans" value={supportsPlans} />
		<input type="hidden" name="can_delegate" value={canDelegate} />

		<div class="flex gap-2 pt-2">
			<Button type="submit" variant="filled">{editingId ? m.common_save() : m.dyn_add()}</Button>
			<Button type="button" variant="outlined" onclick={() => (sheetOpen = false)}>{m.common_cancel()}</Button>
		</div>
	</form>
</BottomSheet>

<style>
	.empty {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 20px 24px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.empty__text {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.empty__text strong {
		color: var(--md-sys-color-on-surface);
	}
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
		padding: 20px;
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
		gap: 14px;
	}
	.card__text {
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
	}
	.card__name {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.25rem;
		font-weight: 700;
	}
	.card__meta {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
		margin: 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.chip {
		padding: 2px 10px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
	}
	.chip--off {
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface-variant);
	}
	.card__intent {
		margin: 0;
		font-size: 0.875rem;
		line-height: 1.5;
		color: var(--md-sys-color-on-surface-variant);
		display: -webkit-box;
		-webkit-box-orient: vertical;
		-webkit-line-clamp: 3;
		line-clamp: 3;
		overflow: hidden;
	}
	.card__actions {
		display: flex;
		gap: 4px;
		margin-top: auto;
	}
	.sheet-title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.5rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.section-label {
		padding: 0 4px;
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}
	.prompt {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.prompt__label {
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.prompt__field {
		padding: 12px 16px;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-large);
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		line-height: 1.5;
		resize: vertical;
	}
	.prompt__field:focus {
		outline: none;
		border: 2px solid var(--md-sys-color-primary);
		padding: 11px 15px;
	}
	.tools {
		display: flex;
		flex-direction: column;
		gap: 10px;
		margin: 4px 0 0;
		padding: 16px;
		border: none;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container);
	}
	.tools__group {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.tools__owner {
		display: flex;
		align-items: center;
		gap: 8px;
		margin: 4px 0 0;
		font-weight: 650;
		color: var(--md-sys-color-on-surface);
	}
</style>
