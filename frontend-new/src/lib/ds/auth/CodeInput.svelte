<script lang="ts">
	import { onMount } from 'svelte';
	import { m } from '$lib/paraglide/messages';

	// Auth / Enter code → Code: six boxes sharing the 380 row 8 apart, three and three with an 8 × 2
	// dash between. 56 high, the digit in mono at the display size. A filled box is edged in
	// border-strong, an empty one in border, the one being typed in accent with the field glow and
	// a 2 × 24 accent caret. Typing moves on, backspace on an empty box goes back, the arrow keys,
	// Home and End move, and a pasted or autofilled code fills every box. A wrong code edges them
	// all in danger. The code is submitted through one hidden input named `name`.
	type Props = {
		/** The digits typed so far; set it to '' to clear the boxes (after a wrong code). */
		value?: string;
		name?: string;
		/** The group's accessible name. */
		label?: string;
		invalid?: boolean;
		disabled?: boolean;
		/** Puts the first empty box in focus on arrival (the code page has nothing else to do). */
		autofocus?: boolean;
		/** Called with the six digits when the last box fills. */
		oncomplete?: (code: string) => void;
		/** The id of what describes the code (the error under it). */
		describedby?: string;
	};
	let { value = $bindable(''), name, label = m.code_label(), invalid = false, disabled = false, autofocus = false, oncomplete, describedby }: Props = $props();

	const LENGTH = 6;
	const id = $props.id();
	const split = (code: string) => Array.from({ length: LENGTH }, (_, index) => code[index] ?? '');
	const initial = () => value;
	let boxes = $state(split(initial().replace(/\D/g, '')));
	const inputs: HTMLInputElement[] = [];

	// Set from outside (cleared after a wrong code): the boxes follow.
	$effect(() => {
		if (value !== boxes.join('')) boxes = split(value.replace(/\D/g, ''));
	});

	function commit(next: string[]) {
		boxes = next;
		value = next.join('');
		for (let index = 0; index < LENGTH; index++) if (inputs[index]) inputs[index].value = next[index];
	}

	function focusAt(index: number) {
		const input = inputs[Math.max(0, Math.min(LENGTH - 1, index))];
		input?.focus();
		input?.select();
	}

	/** Writes `typed` from box `from` on, then moves to the next box to fill. */
	function write(from: number, typed: string) {
		const incoming = typed.replace(/\D/g, '').slice(0, LENGTH - from);
		const next = boxes.slice();
		for (let offset = 0; offset < incoming.length; offset++) next[from + offset] = incoming[offset];
		commit(next);
		focusAt(from + incoming.length);
		if (next.every((digit) => digit !== '')) oncomplete?.(next.join(''));
	}

	function oninput(index: number, event: Event & { currentTarget: HTMLInputElement }) {
		const typed = event.currentTarget.value.replace(/\D/g, '');
		if (typed) return write(index, typed);
		// A letter, or the box emptied: it holds no digit now.
		const next = boxes.slice();
		next[index] = '';
		commit(next);
	}

	function onkeydown(index: number, event: KeyboardEvent & { currentTarget: HTMLInputElement }) {
		const move = { ArrowLeft: index - 1, ArrowRight: index + 1, Home: 0, End: LENGTH - 1 }[event.key];
		if (move !== undefined) {
			event.preventDefault();
			focusAt(move);
		} else if (event.key === 'Backspace' && !boxes[index] && index > 0) {
			event.preventDefault();
			const next = boxes.slice();
			next[index - 1] = '';
			commit(next);
			focusAt(index - 1);
		}
	}

	function onpaste(index: number, event: ClipboardEvent) {
		const pasted = (event.clipboardData?.getData('text') ?? '').replace(/\D/g, '');
		if (!pasted) return;
		event.preventDefault();
		// A whole code goes in from the first box, wherever it was pasted.
		write(pasted.length >= LENGTH ? 0 : index, pasted);
	}

	onMount(() => {
		if (!autofocus) return;
		const empty = boxes.findIndex((digit) => !digit);
		focusAt(empty === -1 ? LENGTH - 1 : empty);
	});
</script>

<div role="group" aria-labelledby="{id}-label" class="flex w-full items-center gap-2">
	<span id="{id}-label" class="sr-only">{label}</span>
	{#each boxes as digit, index (index)}
		{#if index === LENGTH / 2}<span data-part="separator" aria-hidden="true" class="h-0.5 w-2 shrink-0 bg-border-strong"></span>{/if}
		<span class="relative flex min-w-0 flex-1">
			<input
				bind:this={inputs[index]}
				value={digit}
				type="text"
				inputmode="numeric"
				autocomplete={index === 0 ? 'one-time-code' : 'off'}
				aria-label={m.code_digit({ n: String(index + 1) })}
				aria-invalid={invalid ? 'true' : undefined}
				aria-describedby={describedby}
				{disabled}
				oninput={(event) => oninput(index, event)}
				onkeydown={(event) => onkeydown(index, event)}
				onpaste={(event) => onpaste(index, event)}
				onfocus={(event) => event.currentTarget.select()}
				class={[
					'peer h-14 w-full min-w-0 rounded-sm border bg-surface text-center font-mono text-display font-medium text-text caret-transparent outline-none focus:shadow-focus disabled:opacity-50',
					invalid ? 'border-danger' : digit ? 'border-border-strong focus:border-accent' : 'border-border focus:border-accent',
				]}
			/>
			{#if !digit}
				<span
					data-part="caret"
					aria-hidden="true"
					class="pointer-events-none absolute top-1/2 left-1/2 hidden h-6 w-0.5 -translate-x-1/2 -translate-y-1/2 bg-accent peer-focus:block"
				></span>
			{/if}
		</span>
	{/each}
	{#if name}<input type="hidden" {name} {value} />{/if}
</div>
