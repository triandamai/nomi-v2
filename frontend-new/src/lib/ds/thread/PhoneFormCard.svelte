<script lang="ts">
	import { ClipboardList, Pencil } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import IconButton from '../actions/IconButton.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Flow / Mobile Chat — Plan → approve → act, step 3 (MKUyQ): a form the crew prefilled from the
	// chat, folded in the thread, laid out as the pen's Chat / Reminder card (which stays for forms that
	// are reminders): a neutral clipboard-list tile, the form's title at 13 / 500, what's filled in
	// mono, who prefilled it, and the 30 edit button named "Edit {title}" that opens its sheet (ux-lead,
	// gate 3 answer 2).
	type Props = { title: string; values: string; owner?: { agent: CrewId; name: string }; onedit: () => void };
	let { title, values, owner, onedit }: Props = $props();
</script>

<section class="flex w-full flex-wrap items-center gap-3 rounded-md bg-bg px-3.5 py-3 outline -outline-offset-1 outline-border-strong">
	<span class="inline-flex size-9 shrink-0 items-center justify-center rounded-sm bg-surface-2 text-text-2"><Icon icon={ClipboardList} /></span>
	<span class="flex min-w-0 flex-1 basis-40 flex-col gap-0.75">
		<span class="text-ui font-medium text-text">{title}</span>
		{#if values}<span class="font-mono text-label text-text-3">{values}</span>{/if}
	</span>
	<span class="flex items-center gap-2">
		{#if owner}
			<span class="inline-flex items-center gap-1.5">
				<Mascot agent={owner.agent} size={20} />
				<span class="text-caption text-text-3">{owner.name}</span>
			</span>
		{/if}
		<IconButton icon={Pencil} label={m.phone_form_edit({ title })} variant="ghost" iconSize={14} onclick={onedit} />
	</span>
</section>
