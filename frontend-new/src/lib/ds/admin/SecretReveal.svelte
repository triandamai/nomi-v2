<script lang="ts">
	import { Check, Copy, KeyRound } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import { createCopier } from '../actions/copier.svelte';

	// Admin → Secret, shown once: a key or signing secret the backend hands over a single time (a
	// support key, a webhook's new secret). It says so plainly, shows the whole value in mono to
	// select or copy, and goes away when the person says they have it. Built from Notice's warning
	// tone, since losing it means making a new one.
	type Props = {
		title: string;
		secret: string;
		copyLabel: string;
		copiedLabel: string;
		doneLabel?: string;
		ondone?: () => void;
	};
	let { title, secret, copyLabel, copiedLabel, doneLabel, ondone }: Props = $props();

	const copier = createCopier();
</script>

<div role="status" class="flex w-full flex-col gap-2.5 rounded-sm bg-warning-bg px-3 py-2.5">
	<p class="flex items-center gap-2 text-caption text-text">
		<span class="inline-flex text-warning"><Icon icon={KeyRound} size={14} /></span>
		{title}
	</p>
	<code class="block rounded-xs bg-bg px-2.5 py-2 font-mono text-label break-all text-text select-all outline -outline-offset-1 outline-border">{secret}</code>
	<div class="flex flex-wrap items-center gap-2">
		<Button label={copier.copied ? copiedLabel : copyLabel} variant="secondary" iconStart={copier.copied ? Check : Copy} onclick={() => copier.copy(secret)} />
		{#if ondone && doneLabel}<Button label={doneLabel} variant="ghost" onclick={ondone} />{/if}
	</div>
</div>
