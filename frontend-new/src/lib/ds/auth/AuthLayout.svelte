<script lang="ts">
	import type { Snippet } from 'svelte';
	import { CreditCard, Download, Gift } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import { CREW } from '../agents/crew';

	// Auth / Sign in or create (vDaAQ, light hbvNd), Enter code (pUDXz) and First run (jPO7K): the
	// brand panel (760, the crew and what they've been doing) beside the auth side, where the form
	// sits in a 380 column between the language switch and the terms. On a phone, and below 1024,
	// the brand panel goes and the form takes the width (the pen has no phone sign-in; AGENTS.md §2).
	// Widths follow the layout's own box (a container), so it lays out the same in the docs.
	// On purpose: each crew bubble wears its own member's mascot. The pen's Kala bubble wears Rena's
	// and its Dana bubble Maya's (AGENTS.md §9).
	type Props = {
		/** This build's version, shown in mono in the footer ("v1.2.3"). */
		version: string;
		year: number;
		/** False while Nomi is down for maintenance: the status line says so, on a warning dot. */
		healthy?: boolean;
		/** Nomi's line above the form; false leaves it out (a page with its own mascot). */
		greeting?: string | false;
		/** Fills its parent instead of the window, and the form side isn't the page's <main> (the docs). */
		inline?: boolean;
		/** The top row, right-aligned: the language switch. */
		top?: Snippet;
		children: Snippet;
	};
	let { version, year, healthy = true, greeting, inline = false, top, children }: Props = $props();

	const line = $derived(greeting === false ? null : (greeting ?? m.auth_greeting()));
	const shownVersion = $derived(/^\d/.test(version) ? `v${version}` : version);
</script>

{#snippet bubble(agent: 'kala' | 'dana' | 'koda', text: string, when: string)}
	<p class="flex items-center gap-2 rounded-md bg-bg py-2 pr-3 pl-2 whitespace-nowrap shadow-float outline -outline-offset-1 outline-border-strong">
		<Mascot {agent} size={20} />
		<span class="text-ui font-medium text-text">{CREW.find((member) => member.id === agent)?.name}</span>
		<span class="text-ui text-text-2">{text}</span>
		<span class="font-mono text-label text-text-3">{when}</span>
	</p>
{/snippet}

<div class={['@container flex w-full bg-bg text-text', inline ? 'h-full' : 'min-h-dvh lg:h-dvh']}>
	<div class="flex w-full gap-4 p-4">
		<aside
			aria-label={m.auth_brand_title()}
			class="hidden shrink-0 flex-col gap-7 overflow-hidden rounded-md bg-crew-nomi-bg px-10 pt-10 @5xl:flex @5xl:w-1/2 @7xl:w-190"
		>
			<p class="flex items-center gap-2.5">
				<Mascot agent="nomi" size={32} />
				<span class="text-display font-semibold tracking-logo text-text">nomi</span>
			</p>
			<div class="flex flex-col gap-3 pt-6">
				<p class="w-100 max-w-full text-hero font-medium tracking-hero text-text">{m.auth_brand_title()}</p>
				<p class="w-140 max-w-full text-body leading-normal text-text-2">{m.auth_brand_body()}</p>
			</div>
			<ul class="flex flex-wrap items-center gap-4">
				{#each [{ icon: Gift, text: m.auth_trust_free() }, { icon: CreditCard, text: m.auth_trust_card() }, { icon: Download, text: m.auth_trust_export() }] as item (item.text)}
					<li class="flex items-center gap-1.5 text-caption text-text-2">
						<Icon icon={item.icon} size={14} />
						<span>{item.text}</span>
					</li>
				{/each}
			</ul>
			<div class="relative min-h-0 flex-1" role="img" aria-label={m.auth_crew_feed()}>
				<div aria-hidden="true">
					<span class="absolute top-51.5 left-105"><Mascot agent="rena" /></span>
					<span class="absolute top-38 left-140"><Mascot agent="koda" /></span>
					<span class="absolute top-70.5 left-75"><Mascot agent="kala" /></span>
					<span class="absolute top-77 left-128"><Mascot agent="tara" /></span>
					<span class="absolute top-76.5 left-10"><Mascot agent="nomi" /></span>
					<span class="absolute top-89 left-42"><Mascot agent="dana" /></span>
					<span class="absolute top-102 left-100"><Mascot agent="maya" /></span>
					<span class="absolute top-112 -left-4.5"><Mascot agent="personality" /></span>
					<span class="absolute top-115 left-62.5"><Mascot agent="supervisor" /></span>
					<span class="absolute top-107 left-147.5"><Mascot agent="raga" /></span>
					<div class="absolute top-2 left-0">{@render bubble('kala', m.auth_feed_kala(), m.auth_feed_now())}</div>
					<div class="absolute top-16.5 left-9">{@render bubble('dana', m.auth_feed_dana(), m.auth_feed_minutes({ minutes: '2' }))}</div>
					<div class="absolute top-31 left-62.5">{@render bubble('koda', m.auth_feed_koda(), '07:02')}</div>
				</div>
			</div>
		</aside>

		<svelte:element this={inline ? 'section' : 'main'} class="flex min-w-0 flex-1 flex-col items-center overflow-y-auto px-10 py-6 @max-md:px-0 @max-md:py-0">
			<div class="flex min-h-7 w-full items-center justify-end gap-2">{@render top?.()}</div>
			<div class="min-h-6 flex-1" aria-hidden="true"></div>
			<div class="flex w-95 max-w-full flex-col gap-5">
				{#if line}
					<div class="flex items-center gap-2.5">
						<Mascot agent="nomi" size={32} />
						<p class="rounded-md bg-surface-2 px-2.5 py-1.5 text-caption text-text-2">{line}</p>
					</div>
				{/if}
				{@render children()}
			</div>
			<div class="min-h-6 flex-1" aria-hidden="true"></div>
			<footer class="flex w-95 max-w-full flex-col items-center gap-2.5">
				<p class="w-full text-center text-caption text-text-3">{m.auth_terms()}</p>
				<p class="flex items-center gap-2">
					<span class={['size-1.5 shrink-0 rounded-full', healthy ? 'bg-accent' : 'bg-warning']} aria-hidden="true"></span>
					<span class="font-mono text-label text-text-3"
						>{m.auth_status_line({ status: healthy ? m.auth_status_ok() : m.auth_status_maintenance(), version: shownVersion, year: String(year) })}</span
					>
				</p>
			</footer>
		</svelte:element>
	</div>
</div>
