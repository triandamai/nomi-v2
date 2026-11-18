<script lang="ts">
	// Mobile / Action Sheet (Flow / Mobile Projects — Act from the card, step 1: "Stop building …?"):
	// a phone's confirmation, rising over the scrim in two rounded groups 8 apart, 10 from the sides:
	// the question (13 / 600, text-2) and what it means (13 / 1.4, text-3) centred over its actions, each
	// 56 tall at 18 (the destructive one in danger), then the way out on its own (18 / 600). A native
	// modal dialog: Escape and a tap on the scrim keep things as they are.
	type Action = { label: string; danger?: boolean; onclick: () => void };
	type Props = { open?: boolean; title: string; body?: string; actions: Action[]; cancel: string; onclose?: () => void };
	let { open = $bindable(false), title, body, actions, cancel, onclose }: Props = $props();

	let wasOpen = false;
	$effect(() => {
		if (wasOpen && !open) onclose?.();
		wasOpen = open;
	});
	const modal = (dialog: HTMLDialogElement) => dialog.showModal();
</script>

{#if open}
	<dialog
		use:modal
		aria-label={title}
		onclose={() => (open = false)}
		onclick={(event) => event.target === event.currentTarget && (open = false)}
		class="fixed inset-x-0 top-auto bottom-0 m-0 h-auto w-full max-w-none bg-transparent p-0 text-text backdrop:bg-overlay"
	>
		<div class="flex flex-col gap-2 px-2.5 pb-safe">
			<div class="flex flex-col overflow-hidden rounded-2xl bg-surface">
				<div class="flex flex-col items-center gap-1 border-b border-border px-4 pt-3.5 pb-3.25 text-center">
					<p class="text-ui font-semibold text-text-2">{title}</p>
					{#if body}<p class="text-ui leading-cozy text-text-3">{body}</p>{/if}
				</div>
				{#each actions as action, index (action.label)}
					<button
						type="button"
						class={['h-14 w-full cursor-pointer text-phone-action -outline-offset-2', index < actions.length - 1 && 'border-b border-border', action.danger ? 'text-danger' : 'text-text']}
						onclick={() => ((open = false), action.onclick())}>{action.label}</button
					>
				{/each}
			</div>
			<button type="button" class="mb-2 h-14 w-full cursor-pointer rounded-2xl bg-surface text-phone-action font-semibold text-text -outline-offset-2" onclick={() => (open = false)}>{cancel}</button>
		</div>
	</dialog>
{/if}
