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
	import { AlarmClock, Inbox, ListChecks, SquarePen } from '@lucide/svelte';
	import Fab from '$lib/ds/chrome/Fab.svelte';
	import { getLocale } from '$lib/paraglide/runtime';
	import { phoneView } from '$lib/phone.svelte';
	import PhoneAskCard from '$lib/ds/thread/PhoneAskCard.svelte';
	import PhoneTodayList, { type TodayRow } from '$lib/ds/thread/PhoneTodayList.svelte';
	import PhoneCrewRow from '$lib/ds/thread/PhoneCrewRow.svelte';
	import BottomSheet from '$lib/ds/overlays/BottomSheet.svelte';
	import SheetOption from '$lib/ds/overlays/SheetOption.svelte';
	import Mascot from '$lib/ds/agents/Mascot.svelte';

	// App / Home → Primary Column (jNZ30): the greeting, the composer (V55yyr, who it goes to and
	// the model), the starters (Ce004) and the Today strip (BgqD6), 32 apart and centred in the
	// column the pen pads 32 / 56 / 48. A message starts a chat and opens it; the Today strip is
	// left out when the backend can't sum it up.
	let { data } = $props();

	// On a phone Home is its own screen (Flow / Mobile — Home, Inbox & You, step 1 dl13j): the date and
	// the greeting, the ask card, Today as rows, the starters, and the crew. One layout or the other.
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

	// Today on a phone: each column as a row, opening where its things are.
	const TODAY_ICONS = { out: Inbox, reminders: AlarmClock, plans: ListChecks } as const;
	const todayRows = $derived<TodayRow[]>(
		data.home
			? todayColumns(data.home, time.timeZone).map((column) => ({
					icon: TODAY_ICONS[column.key as keyof typeof TODAY_ICONS] ?? Inbox,
					title: column.title,
					detail: column.body,
					count: column.count,
					href: 'href' in column.action && column.action.href ? column.action.href : column.key === 'reminders' ? '/inbox' : '/chats',
				}))
			: [],
	);
	// "WED 21 OCT · 08:14": the day, the date and the month, then the time.
	const today = $derived.by(() => {
		const parts = new Intl.DateTimeFormat(getLocale(), { weekday: 'short', day: 'numeric', month: 'short', timeZone: time.timeZone }).formatToParts(time.now);
		const part = (type: Intl.DateTimeFormatPartTypes) => parts.find((one) => one.type === type)?.value.replace('.', '') ?? '';
		return `${part('weekday')} ${part('day')} ${part('month')} · ${timeOf(time.now, time.timeZone)}`;
	});
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
	<div class="-mx-4 -mt-6 flex flex-col pb-24">
		<div class="flex flex-col gap-1 px-5 pt-1.5 pb-3.5">
			<p class="font-mono text-label tracking-label text-text-3 uppercase">{today}</p>
			<h1 class="text-phone-large font-semibold tracking-phone-large text-text">{title}</h1>
		</div>
		<div class="px-4 pb-4.5">
			<PhoneAskCard
				bind:value={draft}
				bind:field
				placeholder={m.home_placeholder()}
				{to}
				model={modelLabel}
				{busy}
				onsend={start}
				onto={() => (sheet = 'to')}
				onmodel={() => (sheet = 'model')}
			/>
		</div>
		{#if todayRows.length > 0}
			<div class="flex items-center px-5 pb-2">
				<h2 class="flex-1 font-mono text-label tracking-label text-text-3 uppercase">{m.phone_home_today()}</h2>
				<span class="font-mono text-label text-text-3">{m.phone_home_updated({ time: timeOf(data.loadedAt, time.timeZone) })}</span>
			</div>
			<div class="px-4 pb-5"><PhoneTodayList label={m.phone_home_today()} rows={todayRows} /></div>
		{/if}
		{#if picks.length > 0}
			<!-- Starters brings its own TRY label. -->
			<div class="px-4 pb-5"><Starters items={picks} onpick={(index) => start(picks[index].text)} /></div>
		{/if}
		{#if crewMembers.length > 0}
			<div class="flex items-center px-5 pb-2">
				<h2 class="flex-1 font-mono text-label tracking-label text-text-3 uppercase">{m.phone_home_crew()}</h2>
				<span class="font-mono text-label text-text-3">{data.crew.summary.label}</span>
			</div>
			<div class="px-4"><PhoneCrewRow label={m.phone_home_crew()} members={crewMembers} href="/settings/crew" /></div>
		{/if}
	</div>
	<div class="fixed right-4 bottom-14"><Fab label={m.nav_new_chat()} icon={SquarePen} iconSize={22} href="/chat/new" /></div>
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
<div class="flex min-h-full flex-col justify-center gap-8 lg:pb-4">
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
