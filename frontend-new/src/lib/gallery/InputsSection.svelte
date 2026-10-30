<script lang="ts">
	import { GitBranch, Lightbulb } from '@lucide/svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Search from '$lib/ds/inputs/Search.svelte';
	import Chip from '$lib/ds/inputs/Chip.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import Tabs from '$lib/ds/inputs/Tabs.svelte';
	import Toggle from '$lib/ds/inputs/Toggle.svelte';
	import Checkbox from '$lib/ds/inputs/Checkbox.svelte';
	import Composer from '$lib/ds/inputs/Composer.svelte';
	import GallerySection from './GallerySection.svelte';

	const helper = 'Lowercase, numbers and dashes. Becomes your URL.';
</script>

{#snippet board(title: string, note: string, content: import('svelte').Snippet)}
	<div class="flex flex-col gap-4 rounded-md border border-border p-5">
		<p class="text-ui font-medium text-text">{title} <span class="font-mono text-label text-text-3">{note}</span></p>
		{@render content()}
	</div>
{/snippet}

{#snippet caption(text: string)}<span class="font-mono text-label text-text-3">{text}</span>{/snippet}

{#snippet fields()}
	<div class="grid grid-cols-4 gap-6">
		<div class="flex flex-col gap-2">
			<TextField label="Project name" value="budget-keluarga" {helper} mono />
			{@render caption('Field / Default')}
		</div>
		<div class="flex flex-col gap-2">
			<TextField label="Project name" value="budget-kel" {helper} mono data-focus />
			{@render caption('Field / Focus')}
		</div>
		<div class="flex flex-col gap-2">
			<TextField label="Project name" value="Budget Keluarga!" {helper} error="Use lowercase and dashes, e.g. budget-keluarga." mono />
			{@render caption('Field / Error')}
		</div>
		<div class="flex flex-col gap-2">
			<TextField label="Project name" value="salah-daily-tracker" helper="Project name can't change after deploy." mono disabled />
			{@render caption('Field / Disabled')}
		</div>
	</div>
{/snippet}

{#snippet searchAndChips()}
	<div class="flex flex-wrap items-start gap-6">
		<div class="flex w-80 flex-col gap-2"><Search placeholder="Search chats, memory, files" shortcut="⌘K" />{@render caption('Search')}</div>
		<div class="flex flex-col items-start gap-2"><Chip kind="model" label="deepseek-flash" />{@render caption('Chip / Model')}</div>
		<div class="flex flex-col items-start gap-2"><Chip kind="option" label="Default" icon={Lightbulb} />{@render caption('Chip / Option')}</div>
		<div class="flex flex-col items-start gap-2"><Chip kind="agent" label="Nomi" agent="nomi" />{@render caption('Chip / Agent')}</div>
		<div class="flex flex-col items-start gap-2"><Chip kind="provision" label="git repo" icon={GitBranch} />{@render caption('Chip / Provision')}</div>
	</div>
{/snippet}

{#snippet tabsAndToggles()}
	<div class="flex flex-wrap items-start gap-8">
		<div class="flex flex-col items-start gap-2">
			<Segmented
				label="Projects"
				options={[
					{ value: 'all', label: 'All', count: 4 },
					{ value: 'live', label: 'Live', count: 2 },
					{ value: 'building', label: 'Building', count: 1 },
				]}
			/>
			{@render caption('Segmented (composed)')}
		</div>
		<div class="flex w-64 flex-col items-stretch gap-2">
			<Tabs
				label="Run details"
				tabs={[
					{ value: 'transcript', label: 'Transcript' },
					{ value: 'events', label: 'Events' },
				]}
			/>
			{@render caption('Tab / Active · Tab / Inactive')}
		</div>
		<div class="flex flex-col items-start gap-2">
			<span class="flex gap-3"><Toggle label="Weekly summary" checked /><Toggle label="Daily digest" /></span>
			{@render caption('Toggle / On · Off')}
		</div>
		<div class="flex flex-col items-start gap-2">
			<span class="flex gap-4"><Checkbox label="Mask amounts" checked /><Checkbox label="Include names" /></span>
			{@render caption('Check / Checked · Unchecked')}
		</div>
	</div>
{/snippet}

{#snippet composer()}
	<div class="max-w-180">
		<Composer placeholder="Ask Nomi, or hand something to the crew…" agent={{ id: 'nomi', name: 'Nomi' }} model="deepseek-flash" />
	</div>
{/snippet}

<GallerySection
	title="Inputs & selection"
	description="Inputs sit on surface with border-strong. Focus adds the 3px nm-focus ring; errors swap the border and say what to do next."
>
	{@render board('Text field', 'label · field · helper', fields)}

	{@render board('Search & chips', '', searchAndChips)}

	{@render board('Tabs & toggles', '', tabsAndToggles)}

	{@render board('Composer', 'one component for chat, home and project; swap the agent chip and placeholder', composer)}
</GallerySection>
