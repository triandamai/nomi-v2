<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { ArrowUpRight, Calendar, Eye, FileText, HardDrive, Mail, Plug, ShieldCheck, Table, Unplug } from '@lucide/svelte';
	import type { CrewId } from '../agents/crew';
	import BottomSheet from '../overlays/BottomSheet.svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile Chat — Crew, memory & connections, step 5 (SYohQ): a crew member needs an app the
	// person hasn't connected. The sheet names the app and who needs it, says what access means, and
	// starts the connection itself (Google or GitHub sign the person in and come back to the chat:
	// `resume`); any other app opens Connections. The person can answer instead: that closes the sheet
	// and leaves the composer to them. The pen's "Read-only access" and "Read events and free/busy times"
	// are Calendar's own words; a connection isn't always read-only, so the lines say what holds for
	// every app (AGENTS.md §9).
	type Props = {
		open?: boolean;
		agent: CrewId;
		agentName: string;
		/** The app, as the person reads it ("Calendar", "Gmail, Drive"). */
		service: string;
		provider: 'google' | 'github' | 'other';
		/** Google's services asked for, raw (`calendar`). */
		services: string[];
		/** The chat to come back to after signing in. */
		resume: string;
		onanswer: () => void;
	};
	let { open = $bindable(false), agent, agentName, service, provider, services, resume, onanswer }: Props = $props();

	const GLYPH: Record<string, typeof Plug> = { calendar: Calendar, gmail: Mail, drive: HardDrive, docs: FileText, sheets: Table };
	const glyph = $derived(GLYPH[services[0] ?? ''] ?? Plug);
	const title = $derived(provider === 'google' ? m.phone_connect_title_google({ service }) : m.phone_connect_sheet_title({ service }));
</script>

<BottomSheet bind:open {title} subtitle={m.phone_connect_needs({ agent: agentName })}>
	<div class="flex flex-col gap-3.5 px-4 pb-2">
		<div class="flex items-center gap-3">
			<span class="inline-flex size-12 shrink-0 items-center justify-center rounded-lg" style:background-color={`var(--color-crew-${agent}-bg)`} style:color={`var(--crew-${agent}-ink)`}>
				<Icon icon={glyph} size={22} />
			</span>
			<p class="min-w-0 flex-1 text-body leading-cozy text-text">{m.phone_connect_lead()}</p>
		</div>
		<p class="flex items-center gap-2.5 text-body text-text-2"><span class="inline-flex shrink-0"><Icon icon={Eye} size={16} /></span>{m.phone_connect_only()}</p>
		<p class="flex items-center gap-2.5 text-body text-text-2"><span class="inline-flex shrink-0 text-accent"><Icon icon={ShieldCheck} size={16} /></span>{m.phone_connect_asks()}</p>
		<p class="flex items-center gap-2.5 text-body text-text-2"><span class="inline-flex shrink-0"><Icon icon={Unplug} size={16} /></span>{m.phone_connect_disconnect()}</p>
		{#if provider === 'google' || provider === 'github'}
			<form method="POST" action="/settings/connections?/{provider}">
				{#each services as one (one)}<input type="hidden" name="service" value={one} />{/each}
				<input type="hidden" name="resume" value={resume} />
				<button type="submit" class="inline-flex h-11.5 w-full cursor-pointer items-center justify-center gap-2 rounded-lg bg-accent text-phone font-medium text-accent-ink outline-offset-2 focus-visible:outline-2 focus-visible:outline-focus">
					{provider === 'google' ? m.phone_connect_with_google() : m.phone_connect_with_github()}
					<Icon icon={ArrowUpRight} size={16} />
				</button>
			</form>
		{:else}
			<a href="/settings/connections" class="inline-flex h-11.5 w-full items-center justify-center gap-2 rounded-lg bg-accent text-phone font-medium text-accent-ink outline-offset-2 focus-visible:outline-2 focus-visible:outline-focus">
				{m.phone_connect_open_connections()}
				<Icon icon={ArrowUpRight} size={16} />
			</a>
		{/if}
		<button type="button" class="inline-flex h-11.5 w-full cursor-pointer items-center justify-center rounded-lg border border-border-strong text-phone font-medium text-text outline-offset-2 focus-visible:outline-2 focus-visible:outline-focus" onclick={() => ((open = false), onanswer())}>
			{m.phone_connect_answer({ agent: agentName })}
		</button>
	</div>
</BottomSheet>
