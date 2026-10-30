<script lang="ts">
	import { Check, Copy } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import { createCopier } from '../actions/copier.svelte';

	// Mobile / DNS Record: one record per card, so it fits a phone; the value copies in one tap
	// and the state says whether the domain's provider has it yet.
	type Props = { type: string; name: string; value: string; state: 'waiting' | 'verified' | 'failed' };
	let { type, name, value, state }: Props = $props();

	const copier = createCopier();
	const label = $derived(state === 'waiting' ? m.dns_waiting() : state === 'verified' ? m.dns_verified() : m.dns_failed());
</script>

<div class="flex w-full flex-col gap-2.5 rounded-xl bg-surface p-3.5 outline -outline-offset-1 outline-border">
	<p class="flex items-center gap-2">
		<span class="rounded-sm bg-surface-2 px-2 py-0.5 font-mono text-caption text-text">{type}</span>
		<span class="min-w-0 flex-1 text-phone-sm text-text">{m.dns_name({ name })}</span>
		<span
			class={[
				'inline-flex items-center gap-1.25 text-caption',
				state === 'waiting' && 'text-warning',
				state === 'verified' && 'text-accent',
				state === 'failed' && 'text-danger',
			]}><span class="size-1.5 shrink-0 rounded-full bg-current"></span>{label}</span
		>
	</p>
	<div class="flex h-10 items-center gap-2 rounded-md bg-surface-2 pr-1.5 pl-3">
		<span class="min-w-0 flex-1 truncate font-mono text-phone-sm text-text">{value}</span>
		<button
			type="button"
			aria-label={copier.copied ? m.copy_done() : m.dns_copy({ value })}
			class="inline-flex size-10 shrink-0 cursor-pointer items-center justify-center rounded-inner text-text outline-offset-1"
			onclick={() => copier.copy(value)}><Icon icon={copier.copied ? Check : Copy} /></button
		>
	</div>
</div>
