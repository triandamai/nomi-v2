<script lang="ts">
	import type { Snippet } from 'svelte';
	import ProjectDialogPanel from './ProjectDialogPanel.svelte';

	// Design System / Projects → the dialogs, as overlays/Dialog does it: a native modal on the
	// overlay scrim, in the top layer, focus kept inside until it closes (Escape, or `open` set
	// false by its own buttons), at the projects pen's widths. `inline` shows the panel in place,
	// open, as the sheet draws it.
	type Props = {
		open?: boolean;
		inline?: boolean;
		title: string;
		description?: string;
		width?: 420 | 440 | 460 | 520;
		children?: Snippet;
		actions?: Snippet;
		onclose?: () => void;
		onkeydown?: (event: KeyboardEvent) => void;
	};
	let { open = $bindable(false), inline = false, title, description, width = 420, children, actions, onclose, onkeydown }: Props = $props();

	const id = $props.id();
	let wasOpen = false;

	$effect(() => {
		if (wasOpen && !open) onclose?.();
		wasOpen = open;
	});

	function modal(dialog: HTMLDialogElement) {
		dialog.showModal();
	}
</script>

{#if inline}
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div role="group" aria-labelledby="{id}-title" class="contents" {onkeydown}>
		<ProjectDialogPanel {id} {title} {description} {width} {children} {actions} />
	</div>
{:else if open}
	<dialog
		use:modal
		aria-labelledby="{id}-title"
		aria-describedby={description ? `${id}-description` : undefined}
		onclose={() => (open = false)}
		{onkeydown}
		class="m-auto max-w-full bg-transparent p-0 text-text backdrop:bg-overlay"
	>
		<ProjectDialogPanel {id} {title} {description} {width} {children} {actions} />
	</dialog>
{/if}
