<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import CodeToken from '../chat/CodeToken.svelte';
	import type { CodeLine } from '../chat/code';
	import Mascot from '../agents/Mascot.svelte';

	// Flow / Mobile Build — Code, terminal & phone preview (G87UAZ) → a file on a phone, read only:
	// numbered lines in a 14-round card [8,0], each 26 tall [0,12] with its number at 12 in text-3,
	// wrapping (never sideways). A long press picks a line; tapping another then takes the lines
	// between, shown on accent-bg; one floating button asks Koda about them. Without touch: Enter
	// (or Space) on a line picks it, Shift+arrows grow the pick, Escape drops it. The pen's drag
	// handles are the second tap.
	type Props = { lines: CodeLine[]; label: string; onask: (from: number, to: number) => void };
	let { lines, label, onask }: Props = $props();

	let anchor = $state<number | null>(null);
	let focus = $state<number | null>(null);
	const from = $derived(anchor === null ? null : Math.min(anchor, focus ?? anchor));
	const to = $derived(anchor === null ? null : Math.max(anchor, focus ?? anchor));
	const picked = (line: number) => from !== null && to !== null && line >= from && line <= to;
	let buttons: HTMLButtonElement[] = $state([]);

	let pressTimer: ReturnType<typeof setTimeout> | undefined;
	let pressed = false;
	function down(line: number) {
		pressed = false;
		clearTimeout(pressTimer);
		pressTimer = setTimeout(() => {
			pressed = true;
			anchor = line;
			focus = line;
		}, 450);
	}
	const up = () => clearTimeout(pressTimer);
	function tap(line: number) {
		if (pressed) return void (pressed = false);
		// With lines picked a tap takes the lines up to it; a mouse's click picks one at once.
		if (anchor === null) anchor = line;
		focus = line;
	}
	function onkeydown(event: KeyboardEvent, line: number) {
		if (event.key === 'Escape' && anchor !== null) {
			event.preventDefault();
			anchor = focus = null;
			return;
		}
		const step = event.key === 'ArrowDown' ? 1 : event.key === 'ArrowUp' ? -1 : 0;
		if (!step) return;
		const next = Math.min(lines.length, Math.max(1, line + step));
		event.preventDefault();
		if (event.shiftKey) {
			if (anchor === null) anchor = line;
			focus = next;
		}
		buttons[next - 1]?.focus();
	}
	const range = $derived(from === null || to === null ? '' : from === to ? String(from) : `${from}–${to}`);
</script>

<div class="flex w-full flex-col gap-4 pb-18">
	<ol aria-label={label} class="flex w-full flex-col rounded-xl py-2 outline -outline-offset-1 outline-border">
		{#each lines as tokens, index (index)}
			{@const line = index + 1}
			<li>
				<button
					bind:this={buttons[index]}
					type="button"
					aria-label={m.code_line({ n: line })}
					aria-pressed={picked(line)}
					class={['flex min-h-6.5 w-full cursor-pointer items-start gap-2.5 px-3 py-1.25 text-left font-mono text-caption -outline-offset-2 select-none', picked(line) ? 'bg-accent-bg' : '']}
					onpointerdown={() => down(line)}
					onpointerup={up}
					onpointerleave={up}
					oncontextmenu={(event) => event.preventDefault()}
					onclick={() => tap(line)}
					onkeydown={(event) => onkeydown(event, line)}
				>
					<span aria-hidden="true" class="w-4.5 shrink-0 text-right text-text-3">{line}</span>
					<span class="min-w-0 flex-1 break-all whitespace-pre-wrap text-text">{#each tokens as token, at (at)}<CodeToken {token} />{/each}</span>
				</button>
			</li>
		{/each}
	</ol>
	{#if from !== null && to !== null}
		<div class="fixed inset-x-4 bottom-4 z-10 pb-safe">
			<button type="button" class="flex h-13 w-full cursor-pointer items-center gap-2.5 rounded-full bg-surface px-3.5 text-left shadow-float outline -outline-offset-1 outline-border-strong" onclick={() => from !== null && to !== null && onask(from, to)}>
				<Mascot agent="koda" size={20} />
				<span class="min-w-0 flex-1 truncate text-body text-text">{m.code_ask_lines({ name: 'Koda', range })}</span>
			</button>
		</div>
	{/if}
</div>
