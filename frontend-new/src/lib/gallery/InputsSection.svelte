<script lang="ts">
	import { GitBranch, Lightbulb } from '@lucide/svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Search from '$lib/ds/inputs/Search.svelte';
	import Chip from '$lib/ds/inputs/Chip.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import Segment from '$lib/ds/inputs/Segment.svelte';
	import Tab from '$lib/ds/inputs/Tab.svelte';
	import CheckMark from '$lib/ds/inputs/CheckMark.svelte';
	import Toggle from '$lib/ds/inputs/Toggle.svelte';
	import Composer from '$lib/ds/inputs/Composer.svelte';
	import GallerySection from './GallerySection.svelte';
	import GalleryBoard from './GalleryBoard.svelte';

	const helper = 'Lowercase, numbers and dashes. Becomes your URL.';
</script>

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

{#snippet cell(label: string, content: import('svelte').Snippet)}
	<div class="flex flex-col items-start gap-2">{@render content()}{@render caption(label)}</div>
{/snippet}

{#snippet composer()}
	<div class="w-180 max-w-full">
		<Composer placeholder="Ask Nomi, or hand something to the crew…" agent={{ id: 'nomi', name: 'Nomi' }} model="deepseek-flash" />
	</div>
{/snippet}

<GallerySection
	title="Inputs & selection"
	description="Inputs sit on surface with border-strong. Focus adds the 3px nm-focus ring; errors swap the border and say what to do next."
>
	<GalleryBoard title="Text field" note="label · field · helper" nomi>{@render fields()}</GalleryBoard>

	<div class="grid items-start gap-6 lg:grid-cols-2">
		<GalleryBoard title="Search & chips" note="" nomi>
			<div class="flex flex-col gap-4">
				<div class="flex flex-col gap-2"><Search placeholder="Search chats, memory, files" shortcut="⌘K" />{@render caption('Search')}</div>
				<div class="flex flex-wrap items-end gap-4">
					{#snippet model()}<Chip kind="model" label="deepseek-flash" />{/snippet}
					{#snippet option()}<Chip kind="option" label="Default" icon={Lightbulb} />{/snippet}
					{#snippet agent()}<Chip kind="agent" label="Nomi" agent="nomi" />{/snippet}
					{#snippet provision()}<Chip kind="provision" label="git repo" icon={GitBranch} />{/snippet}
					{@render cell('Chip / Model', model)}
					{@render cell('Chip / Option', option)}
					{@render cell('Chip / Agent', agent)}
					{@render cell('Chip / Provision', provision)}
				</div>
			</div>
		</GalleryBoard>

		<GalleryBoard title="Tabs & toggles" note="" nomi>
			<div class="flex flex-col gap-4">
				<div class="flex flex-wrap items-end gap-4">
					{#snippet active()}<Segment label="All" count={4} checked />{/snippet}
					{#snippet inactive()}<Segment label="Live" count={2} />{/snippet}
					{#snippet composed()}<Segmented
							label="Projects"
							options={[
								{ value: 'all', label: 'All', count: 4 },
								{ value: 'live', label: 'Live', count: 2 },
								{ value: 'building', label: 'Building', count: 1 },
							]}
						/>{/snippet}
					{@render cell('Segment / Active', active)}
					{@render cell('Segment / Inactive', inactive)}
					{@render cell('Segmented (composed)', composed)}
				</div>
				<div class="flex flex-wrap items-end gap-4">
					{#snippet tabActive()}<Tab label="Transcript" selected />{/snippet}
					{#snippet tabInactive()}<Tab label="Events" />{/snippet}
					{#snippet on()}<Toggle label="Weekly summary" checked />{/snippet}
					{#snippet off()}<Toggle label="Daily digest" />{/snippet}
					{#snippet checked()}<CheckMark checked aria-label="Mask amounts" />{/snippet}
					{#snippet unchecked()}<CheckMark aria-label="Include names" />{/snippet}
					{@render cell('Tab / Active', tabActive)}
					{@render cell('Tab / Inactive', tabInactive)}
					{@render cell('Toggle / On', on)}
					{@render cell('Toggle / Off', off)}
					{@render cell('Check / Checked', checked)}
					{@render cell('Check / Unchecked', unchecked)}
				</div>
			</div>
		</GalleryBoard>
	</div>

	<GalleryBoard title="Composer" note="one component for chat, home and project; swap the agent chip and placeholder" nomi>{@render composer()}</GalleryBoard>
</GallerySection>
