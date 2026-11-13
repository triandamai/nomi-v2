<script lang="ts">
	import { Info, Plug } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { answersText, formText, type FormField, type ReplyBlock } from '$lib/adapters/blocks';
	import type { CrewId } from '$lib/ds/agents/crew';
	import type { ClarifyAnswer } from '$lib/ds/chat/clarify';
	import ClarifyCard from '$lib/ds/chat/ClarifyCard.svelte';
	import ClarifyAnswered from '$lib/ds/chat/ClarifyAnswered.svelte';
	import ApprovalCard from '$lib/ds/chat/ApprovalCard.svelte';
	import DataTable from '$lib/ds/chat/DataTable.svelte';
	import DecisionCard from '$lib/ds/chat/DecisionCard.svelte';
	import FormCard from '$lib/ds/chat/FormCard.svelte';
	import SelectionCard from '$lib/ds/chat/SelectionCard.svelte';
	import Cart from '$lib/ds/chat/Cart.svelte';
	import Sources from '$lib/ds/chat/Sources.svelte';
	import LinkPreview from '$lib/ds/chat/LinkPreview.svelte';
	import CodeBlock from '$lib/ds/chat/CodeBlock.svelte';
	import Boundary from '$lib/ds/chat/Boundary.svelte';
	import QuickReplies from '$lib/ds/chat/QuickReplies.svelte';
	import Reminder from '$lib/ds/chat/Reminder.svelte';
	import ConnectPrompt from '$lib/ds/chat/ConnectPrompt.svelte';
	import SupervisorStopped from '$lib/ds/chat/SupervisorStopped.svelte';
	import MemoryUsed from '$lib/ds/chat/MemoryUsed.svelte';
	import MemorySaved from '$lib/ds/chat/MemorySaved.svelte';
	import TurnNote from '$lib/ds/thread/TurnNote.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Checkbox from '$lib/ds/inputs/Checkbox.svelte';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import { callJson } from './client';
	import PlanBlock from './PlanBlock.svelte';

	// What a reply carries beyond its text, drawn with the design system's cards (Chat responses:
	// Rich content, Actions, Thinking & planning, Crew behaviour) and acted on. Answers, choices,
	// forms and picks go back as the person's next message, as the backend reads them; an approval
	// is decided through the chat's own endpoint; a reminder is marked done or snoozed. Each card
	// remembers what was done with it on this visit, so it doesn't offer it twice.
	type Props = {
		blocks: ReplyBlock[];
		/** The chat's own endpoints (/chat/:id/api). */
		api: string;
		messageId: string;
		agentName: string;
		language: string;
		timeZone?: string;
		now: Date;
		working: boolean;
		/** Only the latest reply's cards ask for anything. */
		latest: boolean;
		onsend: (text: string) => void;
	};
	let { blocks, api, messageId, agentName, language, timeZone, now, working, latest, onsend }: Props = $props();

	// What was done with each card on this visit, by its key.
	let answered = $state<Record<string, string>>({});
	let decided = $state<Partial<Record<string, 'approved' | 'rejected'>>>({});
	let reminders = $state<Partial<Record<string, 'done' | 'snoozed' | 'gone'>>>({});
	let forms = $state<Record<string, Record<string, string | boolean>>>({});

	function send(key: string, text: string) {
		if (!text) return;
		answered = { ...answered, [key]: text };
		onsend(text);
	}

	const picked = (block: Extract<ReplyBlock, { kind: 'clarify' }>, answers: ClarifyAnswer[]) =>
		block.questions.map((question, index) => {
			const answer = answers[index];
			if (!answer) return [];
			if ('other' in answer) return [answer.other.trim()].filter(Boolean);
			if ('choices' in answer) return answer.choices.map((choice) => question.options[choice].title);
			return [question.options[answer.choice].title];
		});

	async function decide(block: Extract<ReplyBlock, { kind: 'approval' }>, decision: 'approve' | 'deny', remember = false) {
		const result = await callJson(`${api}/messages/${messageId}/approval`, { method: 'PUT', body: { decision, remember } });
		if (!result.ok) return void toasts.show({ kind: 'error', title: m.approval_failed(), detail: result.message || undefined });
		decided = { ...decided, [block.key]: decision === 'approve' ? 'approved' : 'rejected' };
	}

	async function remind(block: Extract<ReplyBlock, { kind: 'reminder' }>, action: 'done' | 'snooze') {
		const result = await callJson(`${api}/reminders/${block.id}`, { method: 'POST', body: action === 'snooze' ? { action, minutes: 30 } : { action } });
		if (result.ok) reminders = { ...reminders, [block.key]: action === 'done' ? 'done' : 'snoozed' };
		else if (result.status === 404) reminders = { ...reminders, [block.key]: 'gone' };
		else toasts.show({ kind: 'error', title: m.reminder_failed() });
	}

	const when = (iso: string) => new Date(iso).toLocaleString(language, { weekday: 'short', day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit', timeZone });
	const value = (key: string, field: FormField) => forms[key]?.[field.key] ?? (field.initial === undefined ? (field.type === 'checkbox' ? false : '') : field.type === 'checkbox' ? field.initial === true : String(field.initial));
	const setValue = (key: string, field: string, next: string | boolean) => (forms = { ...forms, [key]: { ...forms[key], [field]: next } });
</script>

{#each blocks as block (block.key)}
	{#if block.kind === 'clarify'}
		{#if answered[block.key] || !latest}
			<ClarifyAnswered count={block.questions.length} summary={answered[block.key] ?? block.asked.join(' · ')} onopen={() => {}} />
		{:else}
			<ClarifyCard
				agent={block.agent}
				title={block.title}
				questions={block.questions}
				onfinish={(answers) => send(block.key, answersText(block.asked.map((question) => ({ question })), picked(block, answers)))}
				onskipall={() => send(block.key, m.clarify_skip_message())}
			/>
		{/if}
	{:else if block.kind === 'approval'}
		{@const status = decided[block.key] ?? block.status}
		<ApprovalCard
			agent={block.agent}
			title={block.title}
			fields={block.fields}
			status={status === 'cancelled' ? 'rejected' : status}
			outcome={status === 'approved' ? m.approval_approved() : status === 'rejected' ? m.approval_rejected() : status === 'cancelled' ? m.approval_cancelled() : undefined}
			onapprove={status === 'pending' ? ({ remember }) => decide(block, 'approve', remember) : undefined}
			onreject={status === 'pending' ? () => decide(block, 'deny') : undefined}
		/>
	{:else if block.kind === 'plan'}
		<PlanBlock {api} agentSessionId={block.agentSessionId} planId={block.planId} title={block.title} agent={'rena' as CrewId} name={agentName} {working} onstop={() => onsend('stop')} />
	{:else if block.kind === 'table'}
		<DataTable title={block.title} columns={block.columns} rows={block.rows} source={block.source} />
	{:else if block.kind === 'decision'}
		{#if answered[block.key]}
			<TurnNote icon={Info} text={m.block_answered({ answer: answered[block.key] })} />
		{:else}
			<DecisionCard question={block.question} options={block.options} onconfirm={(index) => send(block.key, block.options[index].title)} />
		{/if}
	{:else if block.kind === 'form'}
		{#if answered[block.key]}
			<TurnNote icon={Info} text={m.block_sent()} />
		{:else}
			<FormCard
				agent={block.agent}
				title={block.title}
				note={block.intro}
				submitLabel={block.submitLabel ?? m.form_send()}
				onsubmit={() => send(block.key, formText(block.title, block.fields, Object.fromEntries(block.fields.map((field) => [field.key, value(block.key, field)]))))}
			>
				{#each block.fields as field (field.key)}
					{#if field.type === 'checkbox'}
						<Checkbox label={field.label} checked={value(block.key, field) === true} onchange={(checked) => setValue(block.key, field.key, checked)} />
					{:else}
						<TextField
							label={field.label}
							value={String(value(block.key, field))}
							placeholder={field.placeholder ?? (field.options.length ? field.options.join(' / ') : undefined)}
							required={field.required}
							oninput={(next) => setValue(block.key, field.key, next)}
						/>
					{/if}
				{/each}
			</FormCard>
		{/if}
	{:else if block.kind === 'selection'}
		{#if answered[block.key]}
			<TurnNote icon={Info} text={m.block_answered({ answer: answered[block.key] })} />
		{:else}
			<SelectionCard
				agent={block.agent}
				question={block.question}
				items={block.items}
				selected={block.selected}
				confirmLabel={block.confirmLabel ?? m.selection_confirm()}
				onconfirm={(indices) => send(block.key, indices.map((index) => block.items[index].name).join(', ') || m.selection_none())}
			/>
		{/if}
	{:else if block.kind === 'cart'}
		<Cart title={block.title} store={block.store} items={block.items} subtotal={block.subtotal} delivery={block.delivery} total={block.total} note={block.note} checkoutHref={block.checkoutHref} />
	{:else if block.kind === 'sources'}
		<Sources id="{block.key}-src" sources={block.sources} />
	{:else if block.kind === 'link'}
		<LinkPreview href={block.href} host={block.host} title={block.title} description={block.description} image={block.image} />
	{:else if block.kind === 'code'}
		<CodeBlock code={block.code} language={block.language} file={block.file} />
	{:else if block.kind === 'boundary'}
		<Boundary
			title={block.title}
			reason={block.reason}
			alternatives={block.alternatives.map((alternative) => ({
				agent: alternative.agent,
				title: alternative.title,
				detail: alternative.detail,
				...(alternative.href ? { href: alternative.href, external: true } : alternative.message ? { onclick: () => onsend(alternative.message!) } : {}),
			}))}
		/>
	{:else if block.kind === 'replies'}
		{#if latest && !working && !answered[block.key]}<QuickReplies replies={block.replies} onpick={(reply) => send(block.key, reply)} />{/if}
	{:else if block.kind === 'reminder'}
		{@const state = reminders[block.key]}
		<Reminder
			title={block.title}
			when={state === 'done' ? m.reminder_done() : state === 'snoozed' ? m.reminder_snoozed() : state === 'gone' ? m.reminder_gone() : when(block.dueAt)}
			due={!state && Date.parse(block.dueAt) <= now.getTime()}
			ondone={state ? undefined : () => remind(block, 'done')}
			onsnooze={state ? undefined : () => remind(block, 'snooze')}
		/>
	{:else if block.kind === 'connect'}
		<ConnectPrompt
			agent={block.agent}
			icon={Plug}
			service={block.service}
			why={block.reason === 'key_rejected' ? m.connect_key_rejected({ service: block.service }) : block.reason === 'service_not_allowed' ? m.connect_not_allowed({ service: block.service }) : m.connect_why({ service: block.service })}
			reads={m.connect_reads()}
			href="/settings/connections"
		/>
	{:else if block.kind === 'stopped'}
		<SupervisorStopped tasks={block.tasks} onresume={latest && !working && !answered[block.key] ? () => send(block.key, m.stopped_resume_message()) : undefined} />
	{:else if block.kind === 'voice'}
		<figure class="flex flex-col gap-1.5">
			<audio controls src={block.url} aria-label={m.voice_recording()} class="w-full max-w-120"></audio>
			{#if block.text}<figcaption class="text-caption text-text-3">{block.text}</figcaption>{/if}
		</figure>
	{:else if block.kind === 'memory_used'}
		{#each block.facts as fact (fact)}<MemoryUsed {fact} />{/each}
	{:else if block.kind === 'memory_saved'}
		{#each block.facts as fact (fact)}<MemorySaved {fact} />{/each}
	{:else if block.kind === 'unsupported'}
		<TurnNote icon={Info} text={m.block_unsupported({ view: block.view })} />
	{/if}
{/each}
