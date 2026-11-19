<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import BottomSheet from '../overlays/BottomSheet.svelte';
	import type { MoveTarget } from './ProjectMoveDialog.svelte';

	// Mobile / Move Sheet (FCln5; Flow / Mobile Projects — Card menu, step 5 WbD1H): moving a project
	// to another workspace on a phone, titled "Move <name>" over "To another workspace". [4,16,0,16],
	// 12 apart: the workspaces as a radio group of cards (68 tall, 14 in, 14 corners and a
	// border-strong stroke; a 36 tile on surface-2 with 9 corners, the name at 16 / 500 over its line
	// at 13, a 22 radio stroked 1.5 at the end): the one it's in now at half strength and not
	// choosable, the chosen one stroked text-2 at 2 with the radio's accent ring 7 thick. Under them
	// what changes for whom (the page's ground, 12 corners, 14 in, 14 / 1.45), and Move project: the
	// whole width, 46, accent. The same workspaces and words as the desktop's Move dialog.
	type Props = { open?: boolean; name: string; chats: number; owner?: string; workspaces: MoveTarget[]; value?: string; onmove: (id: string) => void; onclose?: () => void };
	let { open = $bindable(false), name, chats, owner, workspaces, value = $bindable(), onmove, onclose }: Props = $props();

	const current = $derived(workspaces.find((workspace) => workspace.current));
	const chosen = $derived(workspaces.find((workspace) => workspace.id === value && !workspace.current));
</script>

<BottomSheet bind:open title={m.projects_phone_move_title({ name })} subtitle={m.projects_phone_move_sub()} {onclose}>
	<div class="flex flex-col gap-3 px-4 pt-1 pb-2">
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
						'flex h-17 w-full items-center gap-3 rounded-xl p-3.5 text-left outline',
						workspace.current ? 'cursor-default opacity-50' : 'cursor-pointer',
						checked ? '-outline-offset-2 outline-2 outline-text-2' : '-outline-offset-1 outline-border-strong',
					]}
				>
					<span aria-hidden="true" class="inline-flex size-9 shrink-0 items-center justify-center rounded-workspace bg-surface-2 text-body font-semibold text-text">{workspace.name.charAt(0).toUpperCase()}</span>
					<span class="flex min-w-0 flex-1 flex-col gap-0.5">
						<span class="truncate text-phone font-medium text-text">{workspace.name}</span>
						<span class="truncate text-ui text-text-3">{workspace.meta}</span>
					</span>
					<svg aria-hidden="true" viewBox="0 0 22 22" class="size-5.5 shrink-0 fill-none">
						{#if checked}<circle cx="11" cy="11" r="7.5" stroke-width="7" class="stroke-accent" />{:else}<circle cx="11" cy="11" r="10.25" stroke-width="1.5" class="stroke-border-strong" />{/if}
					</svg>
				</button>
			{/each}
		</div>
		{#if chosen}
			<div class="flex flex-col gap-1.5 rounded-lg bg-bg p-3.5" aria-live="polite">
				<p class="text-phone-sm leading-comfy text-text">{m.projects_move_members({ members: chosen.members, workspace: chosen.name, chats, current: current?.name ?? '' })}</p>
				<p class="text-phone-sm leading-comfy text-text-3">{owner ? m.projects_move_owner({ owner }) : chosen.owner ? m.projects_move_new_owner({ owner: chosen.owner, workspace: chosen.name }) : m.projects_move_stays()}</p>
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
