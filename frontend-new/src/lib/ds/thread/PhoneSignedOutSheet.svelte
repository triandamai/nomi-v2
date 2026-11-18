<script lang="ts">
	import { FilePenLine } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import PhoneButton from '../mobile/PhoneButton.svelte';

	// Flow / Mobile Chat — Errors & recovery, scenario 7 step 1 (hAhHg): the sign-in ran out mid-chat.
	// A sheet that can't be dismissed (no close, no Escape, no tap outside: there's nothing to go
	// back to): the grabber and "You've been signed out" (the pen's header with its close switched
	// off), what happened at 15 / 1.5, the draft that's kept on this phone, and Sign in again.
	type Props = { open: boolean; draft?: string; signInHref: string };
	let { open, draft, signInHref }: Props = $props();
	const id = $props.id();

	function modal(dialog: HTMLDialogElement) {
		dialog.showModal();
		dialog.querySelector<HTMLElement>('a')?.focus();
	}
</script>

{#if open}
	<dialog
		use:modal
		role="alertdialog"
		aria-labelledby="{id}-title"
		aria-describedby="{id}-body"
		oncancel={(event) => event.preventDefault()}
		class="fixed inset-x-0 top-auto bottom-0 m-0 h-auto max-h-dvh w-full max-w-none bg-transparent p-0 text-text backdrop:bg-overlay"
	>
		<div class="flex flex-col rounded-t-sheet border-t border-border-strong bg-surface pb-safe">
			<span aria-hidden="true" class="flex h-5 items-center justify-center"><span class="h-1.25 w-9 rounded-full bg-border-strong"></span></span>
			<h2 id="{id}-title" class="px-4 pt-0.5 pb-3 text-phone-title font-semibold text-text">{m.session_title()}</h2>
			<div class="flex flex-col gap-3 px-4 pb-2">
				<p id="{id}-body" class="text-body leading-normal text-text-2">{m.phone_auth_body()}</p>
				{#if draft}
					<p class="flex items-center gap-2.5 rounded-lg bg-bg px-3 py-2.5 outline -outline-offset-1 outline-border">
						<span class="inline-flex shrink-0 text-text-3"><Icon icon={FilePenLine} size={14} /></span>
						<span class="min-w-0 flex-1 truncate text-phone-sm text-text-2">{draft}</span>
						<span class="font-mono text-label text-text-3">{m.phone_auth_kept()}</span>
					</p>
				{/if}
				<PhoneButton label={m.session_sign_in()} href={signInHref} />
			</div>
		</div>
	</dialog>
{/if}
