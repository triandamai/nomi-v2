<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import BottomSheet from '../overlays/BottomSheet.svelte';
	import type { MoveTarget } from './ProjectMoveDialog.svelte';

	// Mobile / Move Sheet (Flow / Mobile Projects — Card menu, step 5): moving a project to another
	// workspace on a phone. The workspaces as a radio group of cards ([12,14], 14 corners, a 32 tile,
	// the name at 16 and its line at 13, a 22 radio at the end): the one it's in now quieter and not
	// choosable, the chosen one stroked text-2 with the radio in accent. Under them what changes for
	// whom (the page's ground, 12 corners), and Move project: the whole width, 46, accent. The same
	// workspaces and words as the desktop's Move dialog.
	type Props = { open?: boolean; name: string; chats: number; owner?: string; workspaces: MoveTarget[]; value?: string; onmove: (id: string) => void; onclose?: () => void };
	let { open = $bindable(false), name, chats, owner, workspaces, value = $bindable(), onmove, onclose }: Props = $props();

	const current = $derived(workspaces.find((workspace) => workspace.current));
	const chosen = $derived(workspaces.find((workspace) => workspace.id === value && !workspace.current));
</script>

<BottomSheet bind:open title={m.projects_move_title({ name })} subtitle={m.projects_phone_move_sub()} {onclose}>
	<div class="flex flex-col gap-3 px-4 pb-2">
		<div role="radiogroup" aria-label={m.workspace_switcher()} class="flex flex-col gap-2">
			{#each workspaces as workspace (workspace.id)}
				{@const checked = workspace.id === chosen?.id}
				<button
					type="button"
					role="radio"
					aria-checked={checked}
					aria-disabled={workspace.current || undefined}
					onclick={() => !workspace.current && (value = workspace.id)}
					class={[
						'flex w-full items-center gap-3 rounded-xl px-3.5 py-3 text-left outline',
						workspace.current ? 'cursor-default opacity-50 outline-border' : 'cursor-pointer',
						checked ? '-outline-offset-2 outline-2 outline-text-2' : ['-outline-offset-1', !workspace.current && 'outline-border-strong'],
					]}
				>
					<span aria-hidden="true" class="inline-flex size-8 shrink-0 items-center justify-center rounded-md bg-surface-2 text-body font-semibold text-text">{workspace.name.charAt(0).toUpperCase()}</span>
					<span class="flex min-w-0 flex-1 flex-col gap-0.5">
						<span class="truncate text-phone text-text">{workspace.name}</span>
						<span class="truncate text-ui text-text-3">{workspace.current ? m.projects_phone_move_current({ meta: workspace.meta }) : workspace.meta}</span>
					</span>
					<span aria-hidden="true" class={['inline-flex size-5.5 shrink-0 items-center justify-center rounded-full border-2', checked ? 'border-accent' : 'border-border-strong']}>
						{#if checked}<span class="size-2.5 rounded-full bg-accent"></span>{/if}
					</span>
				</button>
			{/each}
		</div>
		{#if chosen}
			<div class="flex flex-col gap-1.5 rounded-lg bg-bg p-3.5" aria-live="polite">
				<p class="text-phone-sm text-text">{m.projects_move_members({ members: chosen.members, workspace: chosen.name, chats, current: current?.name ?? '' })}</p>
				<p class="text-phone-sm text-text-3">{owner ? m.projects_move_owner({ owner }) : chosen.owner ? m.projects_move_new_owner({ owner: chosen.owner, workspace: chosen.name }) : m.projects_move_stays()}</p>
			</div>
		{/if}
		<button
			type="button"
			disabled={!chosen}
			class="inline-flex h-11.5 w-full cursor-pointer items-center justify-center rounded-lg bg-accent text-phone font-medium text-accent-ink outline-offset-2 disabled:cursor-default disabled:opacity-50"
			onclick={() => {
				if (!chosen) return;
				open = false;
				onmove(chosen.id);
			}}>{m.projects_move_action()}</button
		>
	</div>
</BottomSheet>
