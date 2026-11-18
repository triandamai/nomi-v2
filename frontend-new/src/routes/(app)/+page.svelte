<script lang="ts">
	import { goto } from '$app/navigation';
	import { m } from '$lib/paraglide/messages';
	import { dayPart, greeting } from '$lib/adapters/time';
	import { crewIdOf, mentionOptions, modelMenu, starters, todayColumns, withMention } from '$lib/adapters/chat';
	import { clock } from '$lib/clock';
	import { chatHeader } from '$lib/chat/header.svelte';
	import { startChat } from '$lib/chat/start';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import type { CrewId } from '$lib/ds/agents/crew';
	import ChatComposer, { type Recipient } from '$lib/ds/thread/ChatComposer.svelte';
	import HomeHero from '$lib/ds/home/HomeHero.svelte';
	import TryChips from '$lib/ds/home/TryChips.svelte';
	import TodayCards, { type TodayCard } from '$lib/ds/home/TodayCards.svelte';
	import CrewCapsule from '$lib/ds/home/CrewCapsule.svelte';
	import CrewDrawer from '$lib/ds/home/CrewDrawer.svelte';

	// Landing Page → Option 3 — Hero Nomi (14:383): the crew capsule under the top bar's search
	// (the crew left the sidebar and the old crew column: it opens a drawer of who's at work); the
	// hero (Nomi, a line from Nomi, the greeting in serif, over the brand glow); the composer with
	// crew peeking over it (who it goes to, the model); the starters as pills; and three Today cards.
	// A message starts a chat and opens it; the Today cards are left out when the backend can't sum
	// the day up.
	let { data } = $props();

	const time = clock();
	const header = chatHeader();
	const title = $derived(greeting(data.person.firstName, dayPart(time.now, time.timeZone)));

	const NOMI: Recipient = { agent: 'nomi', name: 'Nomi' };
	let draft = $state('');
	let field: HTMLTextAreaElement | undefined = $state();
	let modelKey = $state<string | null>(null);
	let busy = $state(false);
	let crewOpen = $state(false);

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

	// Who's on the crew, with a mascot: those at work first, for the capsule's faces.
	const members = $derived(data.crew.rows.map((row) => ({ ...row, active: !!row.state })));
	const faces = $derived<CrewId[]>(
		[...members].sort((a, b) => Number(b.active) - Number(a.active)).flatMap((row) => (row.agent ? [row.agent] : [])),
	);
	// The members peeking over the composer, as drawn, when the person has them.
	const PEEKS: CrewId[] = ['dana', 'kala', 'koda', 'sora'];
	const peeks = $derived(PEEKS.filter((agent) => faces.includes(agent)));

	// Each Today card wears who it's about: who replied while you were out (else Nomi), Kala for
	// reminders, Rena for plans.
	const cards = $derived<TodayCard[]>(
		data.home
			? todayColumns(data.home, time.timeZone).map(({ key, title, body, action: { prompt, ...action } }) => ({
					key,
					title,
					body,
					agent: key === 'out' ? crewIdOf(data.home?.while_you_were_out[0]?.agent) : key === 'reminders' ? 'kala' : 'rena',
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

<!-- The capsule sits under the top bar's search: the shell pads the column 56 across and 32 down,
     the bar 24, so it reaches back 32 to line up with the search's right edge. -->
<div class="flex min-h-full flex-col">
	{#if members.length > 0}
		<div class="flex justify-end lg:-mt-5 lg:-mr-8">
			<CrewCapsule {faces} summary={data.crew.summary} label={m.crew_capsule_label()} open={crewOpen} onclick={() => (crewOpen = true)} />
		</div>
	{/if}
	<div class="mx-auto flex w-full max-w-190 flex-1 flex-col justify-center gap-5 py-6 lg:py-8">
		<HomeHero {title} bubble={m.home_bubble()} {peeks}>
			{#snippet composer()}
				<ChatComposer
					bind:value={draft}
					bind:field
					placeholder={m.home_placeholder()}
					hero
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
			{/snippet}
		</HomeHero>
		<TryChips items={picks} onpick={(index) => start(picks[index].text)} />
		{#if cards.length > 0}<div class="pt-7"><TodayCards {cards} /></div>{/if}
	</div>
</div>

<CrewDrawer bind:open={crewOpen} summary={data.crew.summary} {members} />
