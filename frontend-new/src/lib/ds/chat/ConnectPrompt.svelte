<script lang="ts">
	import type { Component } from 'svelte';
	import { ArrowUpRight, Eye, ShieldCheck, Unplug, type LucideProps } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import type { CrewId } from '../agents/crew';

	// Chat responses → Connect Prompt: a crew member needs a tool to go on. It says who needs it and
	// why, exactly what it can read, that it changes nothing without asking and can be
	// disconnected, and offers a way to go on without it.
	type Props = {
		/** Who needs it (the tile wears their tint). */
		agent: CrewId;
		icon: Component<LucideProps>;
		service: string;
		/** e.g. "Tara needs it to check your weekends". */
		why: string;
		/** What it can read, e.g. "Read your events and free/busy times". */
		reads: string;
		/** How to go on without it, e.g. "Or tell me your free weekends". */
		alternative?: string;
		href: string;
		onlater?: () => void;
	};
	let { agent, icon, service, why, reads, alternative, href, onlater }: Props = $props();
</script>

<section class="flex w-full flex-col overflow-hidden rounded-md border border-border-strong bg-bg">
	<header class="flex items-center gap-2.5 border-b border-border bg-surface px-3.5 py-2.5">
		<span class="inline-flex size-7 shrink-0 items-center justify-center rounded-sm" style="background: var(--crew-{agent}-bg); color: var(--crew-{agent}-ink)"><Icon {icon} size={14} /></span>
		<span class="flex min-w-0 flex-1 flex-col gap-px">
			<span class="text-ui font-medium text-text">{m.connect_title({ service })}</span>
			<span class="font-mono text-label text-text-3">{why}</span>
		</span>
	</header>
	<ul class="flex flex-col gap-2 p-3.5 text-caption text-text-2">
		<li class="flex items-center gap-2"><span class="inline-flex shrink-0"><Icon icon={Eye} size={13} /></span>{reads}</li>
		<li class="flex items-center gap-2"><span class="inline-flex shrink-0 text-accent"><Icon icon={ShieldCheck} size={13} /></span>{m.connect_safe()}</li>
		<li class="flex items-center gap-2"><span class="inline-flex shrink-0"><Icon icon={Unplug} size={13} /></span>{m.connect_disconnect()}</li>
	</ul>
	<footer class="flex flex-wrap items-center gap-2 border-t border-border px-3.5 py-2.5">
		<span class="min-w-0 flex-1 basis-40 text-caption text-text-3">{alternative}</span>
		<span class="ml-auto flex items-center gap-2">
			{#if onlater}<Button label={m.connect_later()} variant="ghost" onclick={onlater} />{/if}
			<Button label={m.connect_go()} {href} external iconEnd={ArrowUpRight} />
		</span>
	</footer>
</section>
