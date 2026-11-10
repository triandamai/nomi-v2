<script lang="ts">
	import { onMount } from 'svelte';
	import { Wrench } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import FocusShell from '$lib/ds/shells/FocusShell.svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import Button from '$lib/ds/actions/Button.svelte';

	// Nomi is down for maintenance (GET /api/maintenance). The pen draws no maintenance screen, so
	// this is FocusShell's one column (AGENTS.md §2): what's happening, the admins' own words when
	// they gave some, until when (in mono, where the person is), and a way to try again.
	let { data } = $props();

	let zone = $state('UTC');
	onMount(() => (zone = Intl.DateTimeFormat().resolvedOptions().timeZone));

	const until = $derived.by(() => {
		if (!data.until) return null;
		const at = new Date(data.until);
		if (Number.isNaN(at.getTime())) return null;
		const time = new Intl.DateTimeFormat(getLocale() === 'id' ? 'id-ID' : 'en-GB', {
			weekday: 'short',
			day: 'numeric',
			month: 'short',
			hour: '2-digit',
			minute: '2-digit',
			hourCycle: 'h23',
			timeZone: zone,
			timeZoneName: 'short',
		}).format(at);
		return m.maintenance_until({ time });
	});
</script>

<svelte:head><title>{m.maintenance_title()} · {m.app_name()}</title></svelte:head>

<FocusShell>
	<main class="flex flex-col items-start gap-3 pt-24">
		<span class="inline-flex text-text-2"><Icon icon={Wrench} size={18} /></span>
		<h1 class="text-display font-medium text-text">{m.maintenance_title()}</h1>
		<p class="text-body text-text-2">{m.maintenance_body()}</p>
		{#if data.message}<p class="text-ui leading-normal whitespace-pre-line text-text-2">{data.message}</p>{/if}
		{#if until}<p class="font-mono text-label text-text-3">{until}</p>{/if}
		<div class="pt-3"><Button label={m.maintenance_retry()} variant="secondary" href="/" /></div>
	</main>
</FocusShell>
