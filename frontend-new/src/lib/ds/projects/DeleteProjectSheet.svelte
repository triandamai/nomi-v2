<script lang="ts">
	import { ChevronRight, Download, Trash2 } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import BottomSheet from '../overlays/BottomSheet.svelte';

	// Mobile / Delete Project Sheet (Flow / Mobile Projects — Card menu, step 7): deleting a project
	// on a phone, typed to confirm. [0,16,8,16], 10 apart: what goes (15 / 1.4, text-2; the pen's "Your code stays in your account
	// for 30 days" isn't so, the backend deletes the files at once: §9); Export repo first
	// (surface-2, 12 corners, [10,12], download, a chevron); the field for the name (48 tall, a 2px
	// danger stroke, 12 corners, mono at 16); and Delete project, 46 tall on danger-bg with its
	// stroke, working only once the name is typed as it is.
	type Props = { open?: boolean; name: string; busy?: boolean; onexport?: () => void; ondelete: () => void; onclose?: () => void };
	let { open = $bindable(false), name, busy = false, onexport, ondelete, onclose }: Props = $props();
	let typed = $state('');
	const ready = $derived(typed.trim() === name);
	$effect(() => {
		if (!open) typed = '';
	});
</script>

<BottomSheet bind:open title={m.projects_delete_title({ name })} subtitle={m.projects_delete_sheet_type()} {onclose}>
	<form
		class="flex flex-col gap-2.5 px-4 pb-2"
		onsubmit={(event) => {
			event.preventDefault();
			if (ready && !busy) ondelete();
		}}
	>
		<p class="text-body leading-cozy text-text-2">{m.projects_delete_body_phone()}</p>
		{#if onexport}
			<button type="button" class="flex items-center gap-2 rounded-lg bg-surface-2 px-3 py-2.5 text-left text-phone-sm text-text outline-offset-2" onclick={onexport}>
				<Icon icon={Download} size={14} /><span class="min-w-0 flex-1">{m.projects_delete_export_first()}</span><span class="inline-flex text-text-3"><Icon icon={ChevronRight} size={14} /></span>
			</button>
		{/if}
		<input
			bind:value={typed}
			aria-label={m.projects_delete_sheet_field({ name })}
			placeholder={name}
			autocomplete="off"
			autocapitalize="off"
			spellcheck="false"
			class="h-12 w-full rounded-lg border-2 border-danger bg-surface px-3.25 font-mono text-phone text-text outline-none placeholder:text-text-3"
		/>
		<button
			type="submit"
			disabled={!ready || busy}
			class="inline-flex h-11.5 w-full cursor-pointer items-center justify-center gap-2 rounded-lg border border-danger-border bg-danger-bg text-phone font-medium text-danger outline-offset-2 disabled:cursor-default disabled:opacity-50"
		>
			{m.projects_delete()}<Icon icon={Trash2} size={16} />
		</button>
	</form>
</BottomSheet>
