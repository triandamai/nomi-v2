<script lang="ts">
	import { Send, X } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';

	// App / Settings / Workspace (wZm9B) → Invite Box: inviting people, on the surface outlined in
	// border-strong, 14 in and 12 apart. The addresses (a comma or Enter makes each a chip in mono,
	// one that isn't an address in the danger colours, with an ✕ to take it back; Backspace in an
	// empty field takes the last), the role as two cards (the chosen one on surface-2 with the
	// stronger edge), and the note with Send. What's still typed is sent too, and stays as typed when
	// the field loses focus: making chips then would move the role cards under a click on its way
	// down. The pen's "Copy invite link" is on each pending invite instead (a link needs an address).
	export type RoleOption = { value: string; title: string; body: string };
	type Props = {
		labels: { field: string; placeholder: string; remove: string; role: string; note: string; send: (count: number) => string };
		roles: RoleOption[];
		role?: string;
		error?: string;
		busy?: boolean;
		onsend?: (emails: string[], role: string) => void;
	};
	let { labels, roles, role = $bindable(roles[0]?.value ?? ''), error, busy = false, onsend }: Props = $props();

	const id = $props.id();
	let chips = $state<string[]>([]);
	let draft = $state('');

	const looksRight = (email: string) => /^[^\s@]+@[^\s@]+\.[^\s@.]+$/.test(email);
	const split = (text: string) => text.split(/[\s,;]+/).map((one) => one.trim()).filter(Boolean);
	const pending = $derived([...chips, ...split(draft).filter((one) => !chips.includes(one))]);

	function keep() {
		const more = split(draft).filter((one) => !chips.includes(one));
		chips = [...chips, ...more];
		draft = '';
	}

	function oninput() {
		if (/[,;\s]$/.test(draft)) keep();
	}

	function onkeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			event.preventDefault();
			keep();
		} else if (event.key === 'Backspace' && !draft && chips.length) chips = chips.slice(0, -1);
	}

	function send() {
		keep();
		if (chips.length) onsend?.(chips, role);
	}

	/** Clears what was sent (the page calls this once the invites went). */
	export function clear() {
		chips = [];
		draft = '';
	}
</script>

<div class="flex flex-col gap-3 rounded-md bg-surface p-3.5 outline -outline-offset-1 outline-border-strong">
	<div
		class={[
			'flex min-h-9.5 flex-wrap items-center gap-1.5 rounded-sm border bg-bg px-1.75 py-1.5 focus-within:border-accent',
			error ? 'border-danger-border' : 'border-border-strong',
		]}
	>
		{#each chips as chip (chip)}
			<span
				data-wrong={looksRight(chip) ? undefined : true}
				class={['inline-flex h-6 items-center gap-1 rounded-xs pr-1 pl-2 font-mono text-label', looksRight(chip) ? 'bg-surface-2 text-text' : 'bg-danger-bg text-danger']}
			>
				{chip}
				<button type="button" aria-label={labels.remove.replace('{email}', chip)} onclick={() => (chips = chips.filter((one) => one !== chip))} class="inline-flex cursor-pointer rounded-xs text-text-3 outline-offset-1 hover:text-text">
					<Icon icon={X} size={12} />
				</button>
			</span>
		{/each}
		<input
			type="text"
			aria-label={labels.field}
			aria-invalid={error ? true : undefined}
			aria-describedby={error ? `${id}-error` : undefined}
			autocomplete="off"
			inputmode="email"
			placeholder={labels.placeholder}
			bind:value={draft}
			{oninput}
			{onkeydown}
			class="min-w-40 flex-1 bg-transparent text-ui text-text outline-none placeholder:text-text-3"
		/>
	</div>
	{#if error}<p id="{id}-error" class="-mt-1.5 text-caption text-danger">{error}</p>{/if}
	<div role="radiogroup" aria-label={labels.role} class="grid gap-2.5 sm:grid-cols-2">
		{#each roles as option (option.value)}
			{@const chosen = option.value === role}
			<label
				class={[
					'flex min-h-13.5 cursor-pointer items-center gap-2.5 rounded-sm px-3 py-2.5 outline -outline-offset-1 focus-within:outline-2 focus-within:outline-focus',
					chosen ? 'bg-surface-2 outline-text-2' : 'outline-border hover:bg-surface-2',
				]}
			>
				<input type="radio" name="{id}-role" value={option.value} bind:group={role} class="sr-only" />
				<span aria-hidden="true" class={['inline-flex size-3.5 shrink-0 items-center justify-center rounded-full outline -outline-offset-1', chosen ? 'outline-text' : 'outline-border-strong']}>
					{#if chosen}<span class="size-1.5 rounded-full bg-text"></span>{/if}
				</span>
				<span class="flex min-w-0 flex-col gap-px">
					<span class="text-ui font-medium text-text">{option.title}</span>
					<span class="text-caption text-text-3">{option.body}</span>
				</span>
			</label>
		{/each}
	</div>
	<div class="flex flex-wrap items-center gap-2.5">
		<span class="min-w-0 flex-1 font-mono text-label text-text-3">{labels.note}</span>
		<Button variant="inverse" iconEnd={Send} label={labels.send(pending.length)} loading={busy} disabled={!pending.length} onclick={send} />
	</div>
</div>
