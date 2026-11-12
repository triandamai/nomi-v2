<script lang="ts">
	import { Rocket } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '../actions/Button.svelte';
	import ProjectDialog from './ProjectDialog.svelte';

	// Design System / Projects → Project / Deploy Dialog (A3CDJ): before a draft's first deploy,
	// what will happen and the facts that come with it, on the page colour (10 / 12 in, 6 apart):
	// a 56 wide key in text-3, the value in mono. 420 wide; Deploy is the inverse main action.
	type Props = {
		open?: boolean;
		inline?: boolean;
		name: string;
		/** How many files Koda builds. */
		files: number;
		/** The address it goes live at. */
		url: string;
		/** Where its data lives ("none yet · add a database later"). */
		data: string;
		/** What it uses of the plan ("1 build · 26 of 30 left in Oct"). */
		uses: string;
		ondeploy: () => void;
		onclose?: () => void;
	};
	let { open = $bindable(false), inline = false, name, files, url, data, uses, ondeploy, onclose }: Props = $props();

	const facts = $derived([
		[m.projects_fact_url(), url],
		[m.projects_fact_data(), data],
		[m.projects_fact_uses(), uses],
	]);
</script>

<ProjectDialog bind:open {inline} {onclose} title={m.projects_deploy_title({ name })} description={m.projects_deploy_body({ files })}>
	<dl class="flex flex-col gap-1.5 rounded-sm bg-bg px-3 py-2.5">
		{#each facts as [key, value] (key)}
			<div class="flex gap-3">
				<dt class="w-14 shrink-0 text-caption text-text-3">{key}</dt>
				<dd class="min-w-0 flex-1 font-mono text-label break-all text-text-2">{value}</dd>
			</div>
		{/each}
	</dl>
	{#snippet actions()}
		<Button variant="secondary" label={m.action_cancel()} onclick={() => (open = false)} />
		<Button
			variant="inverse"
			label={m.projects_action_deploy()}
			iconEnd={Rocket}
			onclick={() => {
				open = false;
				ondeploy();
			}}
		/>
	{/snippet}
</ProjectDialog>
