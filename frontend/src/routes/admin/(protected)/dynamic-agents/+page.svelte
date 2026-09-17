<script lang="ts">
	import { enhance } from '$app/forms';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Card from '$lib/components/m3/Card.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import type { ActionData, PageData } from './$types';
	import type { DynamicAgent } from '$lib/types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	// Grouped by source agent, matching the design spec's admin tool-picker requirement.
	const TOOL_GROUPS: { label: string; tools: { name: string; description: string }[] }[] = [
		{
			label: 'Money',
			tools: [
				{ name: 'list_transactions', description: 'List recent transactions' },
				{ name: 'summarize_budget', description: 'Summarize spending by category' },
			],
		},
		{ label: 'Planning', tools: [{ name: 'create_project', description: 'Create a new project' }] },
		{
			label: 'Personality',
			tools: [
				{ name: 'set_personality', description: 'Set personality' },
				{ name: 'list_personality_versions', description: 'List personality history' },
				{ name: 'rollback_personality', description: 'Roll back personality' },
			],
		},
		{ label: 'Supervisor', tools: [{ name: 'list_recent_agent_activity', description: 'List recent agent activity' }] },
		{
			label: 'Coding',
			tools: [
				{ name: 'write_file', description: 'Write a project file' },
				{ name: 'read_file', description: 'Read a project file' },
				{ name: 'list_files', description: 'List project files' },
				{ name: 'delete_file', description: 'Delete a project file' },
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
		sheetOpen = true;
	}

	function toggleTool(toolName: string, checked: boolean) {
		grantedTools = checked ? [...grantedTools, toolName] : grantedTools.filter((t) => t !== toolName);
	}
</script>

<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Dynamic agents</h1>
<p class="md-body-large mt-2" style="color: var(--md-sys-color-on-surface-variant)">
	Agents defined here run through the same engine as built-in agents, with a curated set of tools.
</p>

<div class="mt-6 space-y-3">
	{#each data.agents as agent (agent.id)}
		<Card variant="outlined" class="p-4">
			<div class="flex items-center justify-between">
				<div>
					<p class="md-title-medium" style="color: var(--md-sys-color-on-surface)">
						{agent.name}
						{#if !agent.is_active}
							<span
								class="md-label-medium ml-2 rounded-full px-2 py-0.5"
								style="background: var(--md-sys-color-error-container); color: var(--md-sys-color-on-error-container)"
							>
								Disabled
							</span>
						{/if}
					</p>
					<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">
						{agent.intent_label} · {agent.granted_tools.length} tool(s)
					</p>
				</div>
				<div class="flex items-center gap-2">
					<Button type="button" variant="text" onclick={() => openEdit(agent)}>Edit</Button>
					<form method="POST" action="?/toggleActive" use:enhance>
						<input type="hidden" name="id" value={agent.id} />
						<Button type="submit" variant="text">{agent.is_active ? 'Disable' : 'Enable'}</Button>
					</form>
				</div>
			</div>
		</Card>
	{/each}
</div>

<div class="mt-6">
	<Button type="button" variant="outlined" onclick={openCreate}>+ Add agent</Button>
</div>

<BottomSheet bind:open={sheetOpen}>
	<h2 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">
		{editingId ? 'Edit agent' : 'Add agent'}
	</h2>

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
		<TextField id="name" name="name" label="Name" bind:value={name} required />
		<TextField id="intent_label" name="intent_label" label="Intent label (one word, unique)" bind:value={intentLabel} required />
		<TextField
			id="intent_description"
			name="intent_description"
			label="Intent description (fed to the classifier)"
			bind:value={intentDescription}
			required
		/>
		<label class="md-body-medium flex flex-col gap-1" style="color: var(--md-sys-color-on-surface)">
			System prompt
			<textarea
				name="system_prompt"
				bind:value={systemPrompt}
				required
				rows="6"
				class="rounded-md border px-3 py-2"
				style="border-color: var(--md-sys-color-outline); background: var(--md-sys-color-surface)"
			></textarea>
		</label>

		<p class="md-label-medium mt-2" style="color: var(--md-sys-color-on-surface-variant)">Granted tools</p>
		{#each TOOL_GROUPS as group (group.label)}
			<p class="md-label-medium mt-1" style="color: var(--md-sys-color-on-surface)">{group.label}</p>
			{#each group.tools as tool (tool.name)}
				<label class="md-body-medium flex items-center gap-2" style="color: var(--md-sys-color-on-surface)">
					<input
						type="checkbox"
						name="granted_tools"
						value={tool.name}
						checked={grantedTools.includes(tool.name)}
						onchange={(e) => toggleTool(tool.name, (e.target as HTMLInputElement).checked)}
					/>
					{tool.name} — {tool.description}
				</label>
			{/each}
		{/each}

		<label class="md-body-medium flex items-center gap-2" style="color: var(--md-sys-color-on-surface)">
			<input type="checkbox" bind:checked={supportsTodos} />
			Supports a live to-do checklist
		</label>
		<input type="hidden" name="supports_todos" value={supportsTodos} />
		<label class="md-body-medium flex items-center gap-2" style="color: var(--md-sys-color-on-surface)">
			<input type="checkbox" bind:checked={supportsPlans} />
			Supports writing versioned plans
		</label>
		<input type="hidden" name="supports_plans" value={supportsPlans} />
		<label class="md-body-medium flex items-center gap-2" style="color: var(--md-sys-color-on-surface)">
			<input type="checkbox" bind:checked={canDelegate} />
			Can delegate to other agents
		</label>
		<input type="hidden" name="can_delegate" value={canDelegate} />

		<div class="flex gap-2 pt-2">
			<Button type="submit" variant="filled">{editingId ? 'Save' : 'Add agent'}</Button>
			<Button type="button" variant="outlined" onclick={() => (sheetOpen = false)}>Cancel</Button>
		</div>
	</form>
</BottomSheet>
