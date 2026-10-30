<script lang="ts">
	import { Check, ChevronLeft, Ellipsis, Rocket } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Mobile / Project Nav: back and the project's menu on 44pt targets, the project in mono with
	// whether it's live, and the publish state as a pill. 52pt tall. The pen draws the pill up to
	// date; with changes waiting it turns mint like the workspace's Publish button.
	type Props = {
		project: string;
		status: string;
		live?: boolean;
		publish: 'idle' | 'ready';
		count?: number;
		onback: () => void;
		onmenu: () => void;
		onpublish?: () => void;
	};
	let { project, status, live = true, publish, count = 0, onback, onmenu, onpublish }: Props = $props();
</script>

<header class="flex h-13 w-full items-center gap-1 bg-bg px-2">
	<button type="button" aria-label={m.mobile_back()} class="inline-flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-full text-text outline-offset-1" onclick={onback}>
		<Icon icon={ChevronLeft} size={22} />
	</button>
	<div class="flex min-w-0 flex-1 flex-col gap-0.5">
		<h1 class="truncate font-mono text-body font-medium text-text">{project}</h1>
		<p class="flex items-center gap-1.5 text-caption text-text-3">
			<span class={['size-1.5 shrink-0 rounded-full', live ? 'bg-accent' : 'bg-text-3']}></span><span class="truncate">{status}</span>
		</p>
	</div>
	<button
		type="button"
		disabled={publish === 'idle'}
		onclick={onpublish}
		class={[
			'inline-flex h-9 shrink-0 cursor-pointer items-center gap-1.5 rounded-2xl px-3.25 text-ui whitespace-nowrap outline-offset-1 disabled:cursor-default',
			publish === 'idle' ? 'border border-border-strong text-text-2' : 'border border-accent bg-accent font-medium text-accent-ink',
		]}
	>
		{#if publish === 'idle'}
			<span class="inline-flex text-text-3"><Icon icon={Check} size={14} /></span>{m.ws_up_to_date()}
		{:else}
			<Icon icon={Rocket} size={14} />{count ? m.ws_publish_count({ count: String(count) }) : m.ws_publish()}
		{/if}
	</button>
	<button type="button" aria-label={m.project_options()} class="inline-flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-full text-text outline-offset-1" onclick={onmenu}>
		<Icon icon={Ellipsis} size={22} />
	</button>
</header>
