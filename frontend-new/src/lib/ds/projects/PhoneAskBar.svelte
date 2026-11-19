<script lang="ts">
	import { ArrowUp } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';

	// Mobile / Ask Bar:
	// 1. Over the workspace preview (Flow / Mobile Build, xWDVb): tapping it goes to chat (onask).
	// 2. At the foot of a project's detail screen (Flow / Mobile Projects, kB2Dt): typing and sending (onsend).
	type Props = {
		placeholder?: string;
		label?: string;
		value?: string;
		onsend?: (text: string) => void;
		onask?: () => void;
	};
	let { placeholder, label, value = $bindable(''), onsend, onask }: Props = $props();

	function send(event?: SubmitEvent) {
		event?.preventDefault();
		const text = value.trim();
		if (!text) return;
		value = '';
		onsend?.(text);
	}
</script>

{#if onsend}
	<form class="border-t border-border bg-bg px-4 pt-2.25 pb-1" onsubmit={send}>
		<div class="flex h-13 items-center gap-2.5 rounded-full bg-surface pr-1.5 pl-3.5 shadow-float outline -outline-offset-1 outline-border-strong">
			<Mascot agent="koda" size={20} />
			<input
				bind:value
				placeholder={placeholder ?? label ?? ''}
				aria-label={placeholder ?? label ?? ''}
				enterkeyhint="send"
				autocomplete="off"
				class="h-full min-w-0 flex-1 bg-transparent text-phone text-text caret-accent outline-none placeholder:text-body placeholder:text-text-3"
			/>
			{#if value.trim()}
				<button type="submit" aria-label={m.composer_send()} class="inline-flex size-10 shrink-0 cursor-pointer items-center justify-center rounded-full bg-surface-2 text-text outline-offset-1">
					<Icon icon={ArrowUp} size={18} />
				</button>
			{/if}
		</div>
	</form>
{:else}
	<button type="button" class="flex h-13 w-full cursor-pointer items-center gap-2.5 rounded-full bg-surface px-3.5 text-left shadow-float outline -outline-offset-1 outline-border-strong focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus" onclick={onask}>
		<Mascot agent="koda" size={20} />
		<span class="min-w-0 flex-1 truncate text-body text-text-3">{label ?? placeholder ?? m.pb_ask_bar()}</span>
	</button>
{/if}
