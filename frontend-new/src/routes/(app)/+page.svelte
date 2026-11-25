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
	import { phoneView } from '$lib/phone.svelte';
	import PhoneAskCard from '$lib/ds/thread/PhoneAskCard.svelte';
	import PhoneHomeHero from '$lib/ds/thread/PhoneHomeHero.svelte';
	import BottomSheet from '$lib/ds/overlays/BottomSheet.svelte';
	import SheetOption from '$lib/ds/overlays/SheetOption.svelte';
	import Mascot from '$lib/ds/agents/Mascot.svelte';

	// App / Home → Primary Column (jNZ30): the greeting, the composer (V55yyr, who it goes to and
	// the model), the starters (Ce004) and the Today strip (BgqD6), 32 apart and centred in the
	// column the pen pads 32 / 56 / 48. A message starts a chat and opens it; the Today strip is
	// left out when the backend can't sum it up.
	let { data } = $props();

	// On a phone Home is its own screen: Nomi and the greeting, the starters, and the ask card at the
	// foot (the person's layout over the pen's dl13j, AGENTS.md §9). One layout or the other.
	const guess = () => data.phone;
	const phone = phoneView(guess());
	let sheet = $state<'to' | 'model' | null>(null);

	const time = clock();
	const header = chatHeader();
	const title = $derived(greeting(data.person.firstName, dayPart(time.now, time.timeZone)));

	const NOMI: Recipient = { agent: 'nomi', name: 'Nomi' };
	let draft = $state('');
	let field: HTMLTextAreaElement | undefined = $state();
	let modelKey = $state<string | null>(null);
	let busy = $state(false);

	const crew = $derived(mentionOptions(data.crew.rows));
	// Who new chats start with (Settings › General), else Nomi.
	const startsWith = (): Recipient => {
		const member = mentionOptions(data.crew.rows).find((option) => option.kind === 'crew' && option.agent === data.chat.startsWith);
		return member?.kind === 'crew' ? { agent: member.agent, name: member.name } : NOMI;
	};
	let to = $state<Recipient>(startsWith());
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

	const crewMembers = $derived(crew.filter((option) => option.kind === 'crew').map((option) => ({ agent: option.agent, name: option.name })));

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

{#if phone.current}
	<!-- Nomi and the greeting over a fading grid, the starters, and the ask card held at the foot
	     above the tab bar, its + opening a new chat (AGENTS.md §9). -->
	<div class="-mx-4 -mt-6 -mb-6 flex min-h-full flex-col">
		<div class="flex flex-1 flex-col justify-center">
			<PhoneHomeHero {title} intro={m.home_intro()} />
			{#if picks.length > 0}
				<!-- Starters brings its own TRY label. -->
				<div class="px-4 pb-6"><Starters items={picks} onpick={(index) => start(picks[index].text)} /></div>
			{/if}
		</div>
		<div class="sticky bottom-0 bg-bg px-4 pt-2 pb-4">
			<PhoneAskCard
				bind:value={draft}
				bind:field
				placeholder={m.home_placeholder()}
				{to}
				model={modelLabel}
				{busy}
				roomy
				start={{ label: m.nav_new_chat(), href: '/chat/new' }}
				onsend={start}
				onto={() => (sheet = 'to')}
				onmodel={() => (sheet = 'model')}
			/>
		</div>
	</div>
	<BottomSheet open={sheet === 'to'} title={m.phone_home_to_title()} onclose={() => sheet === 'to' && (sheet = null)}>
		<SheetOption label="Nomi" chosen={to.name === NOMI.name} onclick={() => ((to = NOMI), (sheet = null))}>
			{#snippet lead()}<Mascot agent="nomi" size={20} live={false} />{/snippet}
		</SheetOption>
		{#each crewMembers.filter((member) => member.name !== NOMI.name) as member (member.agent)}
			<SheetOption label={member.name} chosen={to.name === member.name} onclick={() => (chooseRecipient(member.name), (sheet = null))}>
				{#snippet lead()}<Mascot agent={member.agent} size={20} live={false} />{/snippet}
			</SheetOption>
		{/each}
	</BottomSheet>
	<BottomSheet open={sheet === 'model'} title={m.composer_model_title()} onclose={() => sheet === 'model' && (sheet = null)}>
		{#each models as group (group.group)}
			<p class="px-5 pt-3 pb-1 font-mono text-label tracking-label text-text-3 uppercase">{group.group}</p>
			{#each group.models as option (option.key)}
				<SheetOption label={option.label} detail={option.source} chosen={option.chosen} onclick={() => ((modelKey = option.key), (sheet = null))} />
			{/each}
		{/each}
	</BottomSheet>
{:else}
<!-- The shell pads the column 32 top and bottom; the pen's bottom is 48. -->
<div class="my-auto flex min-h-full flex-col gap-8 lg:pb-4">
	<Greeting {title} intro={m.home_intro()} />
	<ChatComposer
		bind:value={draft}
		bind:field
		placeholder={m.home_placeholder()}
		roomy
		hints
		enterSends={data.chat.enterSends}
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
{/if}
