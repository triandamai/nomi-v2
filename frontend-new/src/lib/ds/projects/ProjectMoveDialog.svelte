<script lang="ts" module>
	import type { CrewId } from '../agents/crew';

	export type MoveTarget = {
		id: string;
		name: string;
		/** "Admin · 5 members", in mono. */
		meta: string;
		/** The initial's tile: a crew tint, or quiet. */
		tone: CrewId | 'plain';
		/** People who would see the project there. */
		members: number;
		/** Where the project is now: shown, not chosen. */
		current?: boolean;
	};
</script>

<script lang="ts">
	import { Check, FolderInput, Globe, Users } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import ProjectDialog from './ProjectDialog.svelte';

	// Design System / Projects → Project / Move Dialog (dEQNS): moving a project to another
	// workspace. 440 wide. The workspaces as a radio group (44 rows, 8 in, a 24 tile, name over mono
	// meta; the one it's in now in text-3 and not choosable; the chosen one filled, checked in
	// accent), then what changes for whom on the page colour (12px lines at 1.4, 12px icons). Move
	// project is the primary action: mint, as the pen draws it.
	type Props = {
		open?: boolean;
		inline?: boolean;
		name: string;
		/** Its chats, which move with it. */
		chats: number;
		/** Who owns the workspace it's in now, and approves the move. */
		owner: string;
		workspaces: MoveTarget[];
		value?: string;
		onmove: (id: string) => void;
		onclose?: () => void;
	};
	let { open = $bindable(false), inline = false, name, chats, owner, workspaces, value = $bindable(), onmove, onclose }: Props = $props();

	let buttons: HTMLButtonElement[] = $state([]);
	const current = $derived(workspaces.find((workspace) => workspace.current));
	const chosen = $derived(workspaces.find((workspace) => workspace.id === value && !workspace.current));
	const choosable = $derived(workspaces.filter((workspace) => !workspace.current));

	function onkeydown(event: KeyboardEvent) {
		const step = event.key === 'ArrowDown' || event.key === 'ArrowRight' ? 1 : event.key === 'ArrowUp' || event.key === 'ArrowLeft' ? -1 : 0;
		if (!step || !choosable.length) return;
		event.preventDefault();
		const at = choosable.findIndex((workspace) => workspace.id === value);
		const next = choosable[(at + step + choosable.length) % choosable.length];
		value = next.id;
		buttons[workspaces.indexOf(next)]?.focus();
	}
</script>

<ProjectDialog bind:open {inline} {onclose} width={440} title={m.projects_move_title({ name })}>
	<div role="radiogroup" aria-label={m.workspace_switcher()} tabindex="-1" {onkeydown} class="flex flex-col gap-0.5">
		{#each workspaces as workspace, index (workspace.id)}
			{@const checked = workspace.id === chosen?.id}
			<button
				bind:this={buttons[index]}
				type="button"
				role="radio"
				aria-checked={checked}
				aria-disabled={workspace.current || undefined}
				tabindex={checked || (!chosen && workspace === choosable[0]) ? 0 : -1}
				onclick={() => !workspace.current && (value = workspace.id)}
				class={[
					'flex h-11 w-full items-center gap-2.5 rounded-xs px-2 text-left outline-offset-1',
					workspace.current ? 'cursor-default' : 'cursor-pointer hover:bg-surface-2',
					checked && 'bg-surface-2',
				]}
			>
				<span
					aria-hidden="true"
					class={['inline-flex size-6 shrink-0 items-center justify-center rounded-sm text-caption font-semibold', workspace.tone === 'plain' && 'bg-surface-2 text-text']}
					style={workspace.tone === 'plain' ? undefined : `background: var(--crew-${workspace.tone}-bg); color: var(--crew-${workspace.tone}-ink)`}
					>{workspace.name.charAt(0).toUpperCase()}</span
				>
				<span class="flex min-w-0 flex-1 flex-col gap-0.5">
					<span class={['truncate text-ui', workspace.current ? 'text-text-3' : 'text-text']}>{workspace.name}</span>
					<span class="truncate font-mono text-label text-text-3">{workspace.meta}</span>
				</span>
				{#if checked}<span class="inline-flex text-accent"><Icon icon={Check} size={14} /></span>{/if}
			</button>
		{/each}
	</div>
	{#if chosen}
		<div class="flex flex-col gap-1.5 rounded-sm bg-bg px-3 py-2.5" aria-live="polite">
			<p class="flex gap-2 text-caption leading-cozy text-text-3">
				<span class="inline-flex shrink-0"><Icon icon={Users} size={12} /></span>
				<span>{m.projects_move_members({ members: chosen.members, workspace: chosen.name, chats, current: current?.name ?? '' })}</span>
			</p>
			<p class="flex gap-2 text-caption leading-cozy text-text-3">
				<span class="inline-flex shrink-0"><Icon icon={Globe} size={12} /></span>
				<span>{m.projects_move_owner({ owner })}</span>
			</p>
		</div>
	{/if}
	{#snippet actions()}
		<Button variant="secondary" label={m.action_cancel()} onclick={() => (open = false)} />
		<Button
			label={m.projects_move_action()}
			iconEnd={FolderInput}
			disabled={!chosen}
			onclick={() => {
				if (!chosen) return;
				open = false;
				onmove(chosen.id);
			}}
		/>
	{/snippet}
</ProjectDialog>
