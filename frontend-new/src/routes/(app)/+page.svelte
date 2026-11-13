<script lang="ts">
	import { goto } from '$app/navigation';
	import { m } from '$lib/paraglide/messages';
	import { dayPart, greeting } from '$lib/adapters/time';
	import { mentionOptions, modelMenu, starters, todayColumns, withMention } from '$lib/adapters/chat';
	import { clock } from '$lib/clock';
	import { clock as timeOf } from '$lib/chat/dates';
	import { chatHeader } from '$lib/chat/header.svelte';
	import { startChat } from '$lib/chat/start';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import Greeting from '$lib/ds/chrome/Greeting.svelte';
	import ChatComposer, { type Recipient } from '$lib/ds/thread/ChatComposer.svelte';
	import Starters from '$lib/ds/thread/Starters.svelte';
	import TodayStrip, { type TodayColumnProps } from '$lib/ds/thread/TodayStrip.svelte';

	// App / Home → Primary Column (jNZ30): the greeting, the composer (V55yyr, who it goes to and
	// the model), the starters (Ce004) and the Today strip (BgqD6), 32 apart and centred in the
	// column the pen pads 32 / 56 / 48. A message starts a chat and opens it; the Today strip is
	// left out when the backend can't sum it up.
	let { data } = $props();

	const time = clock();
	const header = chatHeader();
	const title = $derived(greeting(data.person.firstName, dayPart(time.now, time.timeZone)));

	const NOMI: Recipient = { agent: 'nomi', name: 'Nomi' };
	let draft = $state('');
	let field: HTMLTextAreaElement | undefined = $state();
	let to = $state<Recipient>(NOMI);
	let modelKey = $state<string | null>(null);
	let busy = $state(false);

	const crew = $derived(mentionOptions(data.crew.rows));
	const models = $derived(modelMenu(data.models, modelKey ?? data.models.current?.key ?? null));
	const modelLabel = $derived(models.flatMap((group) => group.models).find((option) => option.chosen)?.label ?? data.models.current?.label);
	const picks = $derived(starters(data.crew.rows));
	const columns = $derived<TodayColumnProps[]>(
		data.home
			? todayColumns(data.home, time.timeZone).map(({ action: { prompt, ...action }, ...column }) => ({
					...column,
					action: prompt ? { label: action.label, onclick: () => ask(prompt) } : action,
				}))
			: [],
	);

	/** Puts a prompt in the field to finish, as Add reminder and Start a plan do. */
	function ask(prompt: string) {
		draft = prompt;
		field?.focus();
	}

	function chooseRecipient(name: string) {
		const member = crew.find((option) => option.kind === 'crew' && option.name === name);
		to = member?.kind === 'crew' ? { agent: member.agent, name: member.name } : NOMI;
	}

	async function start(text: string) {
		if (busy) return;
		busy = true;
		const started = await startChat(to.name === NOMI.name ? text : withMention(text, to.name), modelKey);
		busy = false;
		if (started.kind === 'opened') return void goto(started.href);
		draft = text;
		if (started.kind === 'offline') toasts.show({ kind: 'error', title: m.home_start_failed(), detail: m.home_start_offline() });
		else if (started.kind === 'signed_out') header.signedOutDraft = text;
		else toasts.show({ kind: 'error', title: m.home_start_failed(), detail: started.message || undefined });
	}
</script>

<svelte:head><title>{m.app_name()}</title></svelte:head>

<!-- The shell pads the column 32 top and bottom; the pen's bottom is 48. -->
<div class="flex min-h-full flex-col justify-center gap-8 lg:pb-4">
	<Greeting {title} intro={m.home_intro()} />
	<ChatComposer
		bind:value={draft}
		bind:field
		placeholder={m.home_placeholder()}
		roomy
		hints
		{busy}
		{to}
		{crew}
		model={modelLabel}
		{models}
		onsend={start}
		onto={chooseRecipient}
		onmodel={(key) => (modelKey = key)}
	/>
	{#if picks.length > 0}<Starters items={picks} onpick={(index) => start(picks[index].text)} />{/if}
	{#if columns.length > 0}<TodayStrip {columns} updated={timeOf(data.loadedAt, time.timeZone)} />{/if}
</div>
