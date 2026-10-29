<script lang="ts">
	import { LoaderCircle } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import { STEP_ICON, type StepState } from '../chat/stepState';

	// Build → Preview before deploy: the preview stays empty until there's something real to
	// show. It says where the app will appear and lists the plan's real steps.
	type Step = { label: string; state: Exclude<StepState, 'failed'> };
	let { steps }: { steps: Step[] } = $props();
</script>

<div class="flex h-75 w-full flex-col items-center justify-center gap-5 rounded-md border border-border">
	<div class="flex flex-col items-center gap-1.5 text-center">
		<span class="inline-flex animate-spin text-text-2 motion-reduce:animate-none"><Icon icon={LoaderCircle} size={22} /></span>
		<h3 class="text-body font-medium text-text">{m.preview_building_title()}</h3>
		<p class="text-caption text-text-3">{m.preview_building_meta()}</p>
	</div>
	<ol class="flex w-70 max-w-full flex-col gap-2.5">
		{#each steps as step, index (index)}
			<li class="flex items-center gap-2.5">
				<span class={['inline-flex shrink-0', step.state === 'done' && 'text-accent', step.state === 'running' && 'animate-spin text-text motion-reduce:animate-none', step.state === 'waiting' && 'text-text-3']}>
					<Icon icon={STEP_ICON[step.state]} size={14} />
				</span>
				<span class={['text-ui', step.state === 'waiting' ? 'text-text-3' : 'text-text']}>{step.label}</span>
			</li>
		{/each}
	</ol>
</div>
