<script lang="ts">
	// The crew needs an app the person hasn't connected, a Google service they haven't turned on, or
	// a new key for one that stopped working (ConnectPrompt): who needs it and why, what it can read,
	// and a way to it that comes back to this chat. Not now hides it.
	import type { Component } from 'svelte';
	import { Calendar, Dumbbell, FileText, FolderOpen, Mail, Plug, Sheet, type LucideProps } from '@lucide/svelte';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import ConnectPrompt from '$lib/components/ds/chat/ConnectPrompt.svelte';
	import { CREW } from '$lib/components/ds/agents/crew';
	import { CONNECTORS } from '$lib/connectors';
	import { crewId } from '$lib/crewId';
	import { connectLink, isWorkspaceService, serviceLabel, WORKSPACE_SERVICES } from '$lib/workspace';
	import type { ContentBlock } from '$lib/types';

	type ConnectBlock = Extract<ContentBlock, { kind: 'workspace_connect' | 'connector_connect' }>;
	let { block, agent = null }: { block: ConnectBlock; agent?: string | null } = $props();

	const ICONS: Record<string, Component<LucideProps>> = { gmail: Mail, calendar: Calendar, docs: FileText, drive: FolderOpen, sheets: Sheet, hevy: Dumbbell };

	let dismissed = $state(false);

	const sessionId = $derived(page.params.sessionId ?? null);
	const connector = $derived(block.kind === 'connector_connect' ? CONNECTORS.find((c) => c.id === block.connector) : undefined);
	// Who asked: the card says, else (older cards) the connector's own agent, else whoever wrote the message.
	const who = $derived(crewId(block.agent_type ?? (block.kind === 'connector_connect' ? connector?.agent : 'workspace') ?? agent));
	const name = $derived(CREW.find((member) => member.id === who)?.name ?? 'Nomi');

	const prompt = $derived.by(() => {
		if (block.kind === 'workspace_connect') {
			const first = block.services.find(isWorkspaceService);
			const service = block.services.map(serviceLabel).join(', ');
			return {
				key: first ?? '',
				service,
				why: block.reason === 'service_not_allowed' ? m.connect_why_turn_on({ name, service }) : m.connect_why({ name, service }),
				reads: first ? WORKSPACE_SERVICES[first].detail : m.connectors_google_desc(),
				href: connectLink(block.services, sessionId),
			};
		}
		const service = connector?.name ?? block.connector;
		const params = new URLSearchParams();
		if (sessionId) params.set('resume', sessionId);
		if (block.reason === 'key_rejected') params.set('reason', 'key_rejected');
		const query = params.toString();
		return {
			key: block.connector,
			service,
			why: block.reason === 'key_rejected' ? m.connect_why_key({ name, service }) : m.connect_why({ name, service }),
			reads: connector ? connector.description() : service,
			href: `/connections/${block.connector}${query ? `?${query}` : ''}`,
		};
	});
</script>

{#if !dismissed}
	<ConnectPrompt
		agent={who}
		icon={ICONS[prompt.key] ?? Plug}
		service={prompt.service}
		why={prompt.why}
		reads={prompt.reads}
		href={prompt.href}
		onlater={() => (dismissed = true)}
	/>
{/if}
