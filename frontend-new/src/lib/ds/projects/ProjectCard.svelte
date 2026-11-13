<script lang="ts" module>
	export type ProjectStatus = 'live' | 'building' | 'stopped' | 'failed' | 'draft' | 'importing' | 'attention' | 'down';
</script>

<script lang="ts">
	import { tick } from 'svelte';
	import { Activity, ArrowRight, ArrowUpRight, MessageCircle, Play, Rocket, Square, Wrench, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import Status from '../status/Status.svelte';
	import ProjectCardMenu, { type ProjectActions } from './ProjectCardMenu.svelte';
	import type { CrewId } from '../agents/crew';

	// Design System / Projects → Project / Card: Live (ig0xO), Building (b0OUv), Stopped (EU6Mi),
	// Failed (F6qib), Draft (mwu3Z), Importing (qHlDT), Needs answer (m37KA), Down (hUIVu) and
	// Renaming (JyzIn). 384 × 192, fixed (surface, border-strong inside, 16 in, 12 apart):
	//  · identity: the app tile (36, its crew tint, the initial in the app's serif), name and line,
	//    and ⋯ (the Card Menu);
	//  · a status zone exactly 58 tall, its two lines centred: the status (Status: the live dot,
	//    Koda's mascot while Koda works, a mono word when idle) with step or time at the end, then one
	//    detail line (the live URL to visit). Tinted danger when something broke, warning when Koda
	//    needs an answer, else the page colour;
	//  · a footer under a line drawn inside: chats and time in mono, and the one action: inverse when
	//    it needs you, secondary to open or resume, ghost to stop or cancel.
	// `selected` is the 2px text-2 stroke of the card the detail panel shows (the section's note).
	// Renaming swaps the name for a 20 tall field (accent, 6 in) and the line for its keys. The card
	// answers F2 (rename), ⌘/Ctrl D (duplicate) and ⌘/Ctrl E (export), as its menu says.
	// The description is cut short where the pen lets a long one run past its column (§9).
	type Props = {
		name: string;
		description: string;
		/** The app tile's tint. */
		tone: CrewId;
		status: ProjectStatus;
		/** At the status line's end: the step ("3 of 6"), the deploy ("deploy #6") or the time ("00:22"). */
		meta?: string;
		/** The zone's second line: what Koda is doing, why it failed, what's missing. */
		detail?: string;
		/** The live address (live), shown and linked in place of the detail. */
		url?: string;
		/** How many questions Koda is waiting on (attention). */
		questions?: number;
		/** "2 chats · deploy #15 · 2d ago". */
		footer: string;
		/** The project's workspace: Open (live) goes there. */
		href?: string;
		/** The footer's action in every other state. */
		onaction?: () => void;
		actions?: ProjectActions;
		selected?: boolean;
		renaming?: boolean;
		onrename?: (name: string) => void;
		oncancelrename?: () => void;
	};
	let {
		name,
		description,
		tone,
		status,
		meta,
		detail,
		url,
		questions = 0,
		footer,
		href,
		onaction,
		actions,
		selected = false,
		renaming = false,
		onrename,
		oncancelrename,
	}: Props = $props();

	const initial = () => name;
	let draft = $state(initial());
	let field: HTMLInputElement | undefined = $state();
	const startedRenaming = () => renaming;
	let wasRenaming = startedRenaming();

	$effect(() => {
		if (renaming && !wasRenaming) {
			draft = name;
			tick().then(() => field?.select());
		}
		wasRenaming = renaming;
	});

	const label = $derived(
		{
			live: m.projects_status_live(),
			building: m.projects_status_building(),
			stopped: m.projects_status_stopped(),
			failed: m.projects_status_failed(),
			draft: m.projects_status_draft(),
			importing: m.projects_status_importing(),
			attention: questions === 1 ? m.projects_status_question() : m.projects_status_questions({ count: questions }),
			down: m.projects_status_down(),
		}[status],
	);
	const kind = $derived(
		({ live: 'live', building: 'working', importing: 'working', stopped: 'idle', draft: 'idle', failed: 'failed', down: 'failed', attention: 'attention' } as const)[status],
	);
	const action = $derived(
		(
			{
				live: { label: m.projects_action_open(), variant: 'secondary', iconEnd: ArrowRight },
				building: { label: m.projects_action_stop(), variant: 'ghost', iconStart: Square },
				stopped: { label: m.projects_action_resume(), variant: 'secondary', iconEnd: Play },
				failed: { label: m.projects_action_fix(), variant: 'inverse', iconEnd: Wrench },
				draft: { label: m.projects_action_deploy(), variant: 'secondary', iconEnd: Rocket },
				importing: { label: m.action_cancel(), variant: 'ghost', iconStart: X },
				attention: { label: m.projects_action_answer(), variant: 'inverse', iconEnd: MessageCircle },
				down: { label: m.projects_action_check(), variant: 'inverse', iconEnd: Activity },
			} as const
		)[status],
	);

	function onkeydown(event: KeyboardEvent) {
		if (renaming || event.target instanceof HTMLInputElement) return;
		const mod = event.metaKey || event.ctrlKey;
		const run = event.key === 'F2' ? actions?.onrename : mod && event.key.toLowerCase() === 'd' ? actions?.onduplicate : mod && event.key.toLowerCase() === 'e' ? actions?.onexport : undefined;
		if (!run) return;
		event.preventDefault();
		run();
	}

	function onfieldkey(event: KeyboardEvent) {
		if (event.isComposing) return;
		if (event.key === 'Enter') {
			event.preventDefault();
			const next = draft.trim();
			if (next && next !== name) onrename?.(next);
			else oncancelrename?.();
		} else if (event.key === 'Escape') {
			event.preventDefault();
			draft = name;
			oncancelrename?.();
		}
	}
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<article
	aria-label={name}
	{onkeydown}
	class={[
		'flex h-48 w-96 max-w-full shrink-0 flex-col gap-3 rounded-md bg-surface p-4 outline',
		selected ? '-outline-offset-2 outline-2 outline-text-2' : '-outline-offset-1 outline-border-strong',
	]}
>
	<div class="flex h-9 items-center gap-3">
		<span class="inline-flex size-9 shrink-0 items-center justify-center rounded-sm" style="background: var(--crew-{tone}-bg)">
			<span aria-hidden="true" class="font-app text-body font-semibold" style="color: var(--crew-{tone}-ink)">{name.charAt(0).toUpperCase()}</span>
		</span>
		<div class="flex min-w-0 flex-1 flex-col gap-0.5">
			{#if renaming}
				<span class="flex h-5 items-center rounded-sm border border-accent bg-bg px-1.25">
					<input
						bind:this={field}
						bind:value={draft}
						onkeydown={onfieldkey}
						aria-label={m.projects_rename_label()}
						spellcheck="false"
						class="min-w-0 flex-1 bg-transparent text-ui font-medium text-text caret-accent outline-none"
					/>
				</span>
				<span class="truncate font-mono text-label text-text-3">{m.projects_rename_hint()}</span>
			{:else}
				<h3 class="truncate text-ui font-medium text-text">{name}</h3>
				<p class="truncate text-caption text-text-3">{description}</p>
			{/if}
		</div>
		{#if actions}<ProjectCardMenu {name} {actions} live={status === 'live'} />{/if}
	</div>
	<div
		data-testid="status-zone"
		class={[
			'flex h-14.5 shrink-0 flex-col justify-center gap-1.5 rounded-sm px-3',
			status === 'failed' || status === 'down' ? 'bg-danger-bg' : status === 'attention' ? 'bg-warning-bg' : 'bg-bg',
		]}
	>
		<div class="flex items-center justify-between gap-3">
			<Status {kind} {label} agent="koda" />
			{#if meta}<span class="shrink-0 font-mono text-label text-text-3">{meta}</span>{/if}
		</div>
		{#if url}
			<a
				href="https://{url}"
				target="_blank"
				rel="noopener noreferrer"
				class="flex min-w-0 items-center gap-1.5 rounded-xs font-mono text-label text-text-3 outline-offset-2 hover:text-text-2"
			>
				<span class="min-w-0 flex-1 truncate">{url}</span>
				<Icon icon={ArrowUpRight} size={12} />
			</a>
		{:else if detail}
			<p class={['truncate font-mono text-label', status === 'failed' || status === 'down' ? 'text-text-2' : 'text-text-3']}>{detail}</p>
		{/if}
	</div>
	<div class="flex items-center gap-2 border-t border-border pt-2.75">
		<p class="min-w-0 flex-1 truncate font-mono text-label text-text-3">{footer}</p>
		{#if status === 'live' && href}
			<Button variant="secondary" label={action.label} iconEnd={ArrowRight} {href} />
		{:else if onaction}
			<Button variant={action.variant} label={action.label} iconStart={'iconStart' in action ? action.iconStart : undefined} iconEnd={'iconEnd' in action ? action.iconEnd : undefined} onclick={onaction} />
		{/if}
	</div>
</article>
