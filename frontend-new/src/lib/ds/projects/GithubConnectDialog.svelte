<script lang="ts">
	import { Check, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import GithubMark from '../foundations/GithubMark.svelte';
	import Button from '../actions/Button.svelte';
	import ProjectDialog from './ProjectDialog.svelte';

	// Design System / Projects → Project / GitHub Connect Dialog (s3mLi): before Nomi is sent to
	// GitHub, what it will and won't do with the access, on the page colour (12 in, 8 apart; a mint
	// check or a text-3 × at 14, the line at 1.4). 420 wide; Continue on GitHub is the inverse main
	// action.
	type Props = { open?: boolean; inline?: boolean; oncontinue: () => void; onclose?: () => void };
	let { open = $bindable(false), inline = false, oncontinue, onclose }: Props = $props();

	const lines = $derived([
		{ can: true, text: m.projects_github_read() },
		{ can: true, text: m.projects_github_push() },
		{ can: false, text: m.projects_github_never() },
	]);
</script>

<ProjectDialog bind:open {inline} {onclose} title={m.projects_github_title()} description={m.projects_github_body()}>
	<ul aria-label={m.projects_github_scope()} class="flex flex-col gap-2 rounded-sm bg-bg p-3">
		{#each lines as line (line.text)}
			<li class="flex gap-2.5">
				<span class={['inline-flex shrink-0', line.can ? 'text-accent' : 'text-text-3']}><Icon icon={line.can ? Check : X} size={14} /></span>
				<span class="min-w-0 flex-1 text-caption leading-cozy text-text-2">{line.text}</span>
			</li>
		{/each}
	</ul>
	{#snippet actions()}
		<Button variant="secondary" label={m.action_cancel()} onclick={() => (open = false)} />
		<Button
			variant="inverse"
			label={m.projects_github_continue()}
			iconEnd={GithubMark}
			onclick={() => {
				open = false;
				oncontinue();
			}}
		/>
	{/snippet}
</ProjectDialog>
