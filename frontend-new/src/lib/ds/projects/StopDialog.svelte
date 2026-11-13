<script lang="ts">
	import { Square } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '../actions/Button.svelte';
	import ProjectDialog from './ProjectDialog.svelte';

	// Design System / Projects → Project / Stop Dialog (m7QPG7): before stopping a build, what
	// stopping keeps (the finished file, the steps done, nothing deployed) and where Resume picks up.
	// 420 wide; Keep building (secondary) closes it, Stop is the inverse main action.
	type Props = {
		open?: boolean;
		inline?: boolean;
		name: string;
		/** Steps finished so far. */
		done: number;
		/** The step Resume starts from. */
		next: number;
		onstop: () => void;
		onclose?: () => void;
	};
	let { open = $bindable(false), inline = false, name, done, next, onstop, onclose }: Props = $props();
</script>

<ProjectDialog bind:open {inline} {onclose} title={m.projects_stop_title({ name })} description={m.projects_stop_body({ done, next })}>
	{#snippet actions()}
		<Button variant="secondary" label={m.projects_keep_building()} onclick={() => (open = false)} />
		<Button
			variant="inverse"
			label={m.projects_action_stop()}
			iconEnd={Square}
			onclick={() => {
				open = false;
				onstop();
			}}
		/>
	{/snippet}
</ProjectDialog>
