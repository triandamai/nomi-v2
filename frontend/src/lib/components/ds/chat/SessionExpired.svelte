<script lang="ts">
	import { ArrowRight, FilePenLine, LogIn } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';

	// Chat responses → Session Expired: the sign-in ran out mid-chat. Nothing is lost: the unsent
	// message stays on this device and goes out after signing in; the crew keeps working.
	type Props = { draft?: string; signInHref: string };
	let { draft, signInHref }: Props = $props();
</script>

<section class="flex w-105 max-w-full flex-col overflow-hidden rounded-md bg-surface outline -outline-offset-1 outline-border-strong shadow-float">
	<div class="flex flex-col gap-2.5 p-5">
		<span class="inline-flex text-text-2"><Icon icon={LogIn} size={18} /></span>
		<h2 class="text-body font-medium text-text">{m.session_title()}</h2>
		<p class="text-ui leading-normal text-text-2">{m.session_body()}</p>
		{#if draft}
			<p class="flex items-center gap-2 rounded-sm bg-bg px-2.5 py-2 outline -outline-offset-1 outline-border">
				<span class="inline-flex shrink-0 text-text-3"><Icon icon={FilePenLine} size={12} /></span>
				<span class="min-w-0 flex-1 truncate text-caption text-text-2">{draft}</span>
				<span class="font-mono text-label text-text-3">{m.session_draft_kept()}</span>
			</p>
		{/if}
	</div>
	<footer class="flex justify-end border-t border-border px-5 pt-2.75 pb-3">
		<Button label={m.session_sign_in()} href={signInHref} iconEnd={ArrowRight} />
	</footer>
</section>
