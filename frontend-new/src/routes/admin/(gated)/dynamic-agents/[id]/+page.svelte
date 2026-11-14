<script lang="ts">
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { can } from '$lib/admin/access';
	import { MOTIONS, SHAPES, TONES } from '$lib/admin/agents-form';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FormSection from '$lib/ds/admin/FormSection.svelte';
	import Select from '$lib/ds/admin/Select.svelte';
	import TextArea from '$lib/ds/admin/TextArea.svelte';
	import Notice from '$lib/ds/admin/Notice.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Checkbox from '$lib/ds/inputs/Checkbox.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';
	import Outcome from '$lib/admin/ui/Outcome.svelte';

	// Custom agents → an agent (agents: write; view shows it): its name, what it's for (Nomi reads
	// it to hand it questions), how it works, the tools it may use, what else it can do, and how its
	// mascot looks. Without the tool list, its tools are sent as they were.
	let { data, form } = $props();

	const canWrite = $derived(can(data.access, 'agents', 'write'));
	const existing = $derived(data.agent?.ok ? data.agent.data : null);
	const options = (list: readonly string[]) => list.map((value) => ({ value, label: value }));
</script>

{#snippet save()}<Button label={existing ? m.admin_save() : m.admin_agent_create()} variant="inverse" type="submit" />{/snippet}

{#if data.agent && !data.agent.ok}
	<PageHeader title={m.admin_page_dynamic_agents()} back={{ href: '/admin/dynamic-agents', label: m.admin_agent_back() }} />
	<LoadFailed title={m.admin_agent_failed()} problem={data.agent.problem} />
{:else}
	<PageHeader title={existing?.name ?? m.admin_agent_new()} back={{ href: '/admin/dynamic-agents', label: m.admin_agent_back() }} />
	<form method="POST" action="?/save" use:enhance={() => ({ update }) => update({ reset: false })} class="flex flex-col gap-6">
		<FormSection title={m.admin_agent_section()} description={m.admin_agent_section_desc()}>
			<div class="grid gap-4 sm:grid-cols-2">
				<TextField label={m.admin_agent_name()} name="name" value={existing?.name ?? ''} required disabled={!canWrite} autocomplete="off" />
				<TextField label={m.admin_agent_intent_label()} name="intent_label" value={existing?.intent_label ?? ''} required disabled={!canWrite} autocomplete="off" />
				<div class="sm:col-span-2"><TextArea label={m.admin_agent_intent_description()} name="intent_description" value={existing?.intent_description ?? ''} required disabled={!canWrite} /></div>
				<div class="sm:col-span-2"><TextArea label={m.admin_agent_prompt()} name="system_prompt" value={existing?.system_prompt ?? ''} helper={m.admin_agent_prompt_helper()} required disabled={!canWrite} /></div>
			</div>
		</FormSection>

		<FormSection title={m.admin_agent_tools()}>
			{#if data.tools.length > 0}
				<div class="grid gap-2 sm:grid-cols-3">
					{#each data.tools as tool (tool)}
						<Checkbox label={tool} name="tool_{tool}" checked={existing?.granted_tools.includes(tool) ?? false} disabled={!canWrite} />
					{/each}
				</div>
			{:else}
				<Notice kind="info" text={m.admin_agent_tools_none()} />
				{#each existing?.granted_tools ?? [] as tool (tool)}<input type="hidden" name="tool_{tool}" value="on" />{/each}
			{/if}
			<div class="flex flex-col gap-2">
				<span class="text-ui font-medium text-text">{m.admin_agent_abilities()}</span>
				<div class="flex flex-wrap gap-4">
					<Checkbox label={m.admin_agent_todos()} name="supports_todos" checked={existing?.supports_todos ?? false} disabled={!canWrite} />
					<Checkbox label={m.admin_agent_plans()} name="supports_plans" checked={existing?.supports_plans ?? false} disabled={!canWrite} />
					<Checkbox label={m.admin_agent_delegate()} name="can_delegate" checked={existing?.can_delegate ?? false} disabled={!canWrite} />
				</div>
			</div>
		</FormSection>

		<FormSection title={m.admin_agent_look()} description={m.admin_agent_look_desc()} footer={canWrite ? save : undefined}>
			<div class="grid gap-4 sm:grid-cols-3">
				<Select label={m.admin_agent_shape()} name="shape" value={existing?.shape ?? SHAPES[0]} options={options(SHAPES)} mono disabled={!canWrite} />
				<Select label={m.admin_agent_tone()} name="tone" value={existing?.tone ?? TONES[0]} options={options(TONES)} mono disabled={!canWrite} />
				<Select label={m.admin_agent_motion()} name="motion" value={existing?.motion ?? MOTIONS[0]} options={options(MOTIONS)} mono disabled={!canWrite} />
			</div>
			<Outcome {form} action="save" done={m.admin_agent_saved()} failed={m.admin_agent_save_failed()} />
		</FormSection>
	</form>
{/if}
