<script lang="ts">
	import { ArrowRight, Coffee } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import Mascot from '../agents/Mascot.svelte';

	// Build together → Invite email: what the invited person gets. Who invited them to which
	// project, what they'll be able to do, the project, one clear button, and where it was sent.
	type Props = { from: string; project: string; detail?: string; role: 'build' | 'prompt' | 'view'; href: string; sentTo: string; expires: string };
	let { from, project, detail, role, href, sentTo, expires }: Props = $props();

	const first = $derived(from.split(/\s+/)[0]);
	const body = $derived(
		role === 'prompt' ? m.invite_body_prompt({ agent: 'Koda', from: first }) : role === 'build' ? m.invite_body_build({ agent: 'Koda', from: first }) : m.invite_body_view({ from: first }),
	);
</script>

<article class="flex w-130 max-w-full flex-col gap-4.5 rounded-md bg-surface p-7 outline -outline-offset-1 outline-border">
	<p class="flex items-center gap-2"><Mascot agent="nomi" size={20} /><span class="text-ui font-semibold text-text">Nomi</span></p>
	<h2 class="text-heading font-semibold text-text">{m.invite_title({ from, project })}</h2>
	<p class="text-ui leading-normal text-text-2">{body}</p>
	<div class="flex items-center gap-3 rounded-sm p-3 outline -outline-offset-1 outline-border">
		<span class="inline-flex size-9 shrink-0 items-center justify-center rounded-xs bg-accent-bg text-accent"><Icon icon={Coffee} /></span>
		<span class="flex min-w-0 flex-1 flex-col gap-0.5">
			<span class="font-mono text-ui text-text">{project}</span>
			{#if detail}<span class="font-mono text-label text-text-3">{detail}</span>{/if}
		</span>
	</div>
	<span><Button label={m.invite_join()} {href} iconEnd={ArrowRight} /></span>
	<p class="font-mono text-label text-text-3">{m.invite_foot({ email: sentTo, expires })}</p>
</article>
