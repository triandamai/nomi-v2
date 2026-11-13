<script lang="ts">
	import { ArrowUp } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Mascot from '../agents/Mascot.svelte';
	import IconButton from '../actions/IconButton.svelte';

	// Design System / Projects → Project / Ask Field (oZ6Le): asking Koda for a change from the
	// detail panel, which starts a new chat in the project. 340 wide on surface (12 / 10 in, 8
	// apart): Koda and the question (13 at 1.4), then where it goes in mono and Send (the accent
	// icon button, off until there's something to send). The stroke is accent while it holds a
	// question or has focus. Enter sends, Shift+Enter starts a new line.
	type Props = { project: string; value?: string; onask: (text: string) => void };
	let { project, value = $bindable(''), onask }: Props = $props();

	const text = $derived(value.trim());

	function ask(event?: SubmitEvent) {
		event?.preventDefault();
		if (!text) return;
		onask(text);
		value = '';
	}

	function onkeydown(event: KeyboardEvent) {
		if (event.key !== 'Enter' || event.shiftKey || event.isComposing) return;
		event.preventDefault();
		ask();
	}

	function grow(element: HTMLTextAreaElement) {
		$effect(() => {
			void value;
			element.style.height = 'auto';
			element.style.height = `${Math.min(element.scrollHeight, 90)}px`;
		});
	}
</script>

<form
	aria-label={m.projects_ask_label({ name: project })}
	onsubmit={ask}
	class={['flex w-85 max-w-full flex-col gap-2 rounded-md bg-surface px-3 py-2.5 outline -outline-offset-1 focus-within:outline-accent', value ? 'outline-accent' : 'outline-border-strong']}
>
	<div class="flex gap-2">
		<Mascot agent="koda" size={20} />
		<textarea
			bind:value
			use:grow
			{onkeydown}
			rows="1"
			aria-label={m.projects_ask_label({ name: project })}
			class="min-w-0 flex-1 resize-none bg-transparent text-ui leading-cozy text-text caret-accent outline-none placeholder:text-text-3"
		></textarea>
	</div>
	<div class="flex items-center justify-between gap-2">
		<span class="truncate font-mono text-label text-text-3">{m.projects_ask_hint({ name: project })}</span>
		<IconButton icon={ArrowUp} label={m.composer_send()} variant="send" type="submit" disabled={!text} />
	</div>
</form>
