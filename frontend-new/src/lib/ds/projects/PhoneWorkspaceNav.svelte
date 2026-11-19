<script module lang="ts">
	/** The publish pill, by the deploy overview and the person's role (Mobile Build state machine). */
	export type PhonePill =
		| { kind: 'none' }
		| { kind: 'idle' }
		| { kind: 'ready'; count: number; onpublish: () => void }
		| { kind: 'publishing'; elapsed: string }
		| { kind: 'failed'; onretry?: () => void }
		| { kind: 'ask'; busy?: boolean; onask: () => void }
		| { kind: 'asked'; onopen: () => void };
</script>

<script lang="ts">
	import { Check, ChevronLeft, CircleAlert, Ellipsis, Hourglass, LoaderCircle, Rocket } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile Build → Mobile / Project Nav (AJXze) with every state of its publish pill: back and
	// the project's ··· on 44pt targets, the project in mono with whether it's live and which deploy,
	// and the pill. The pen draws Up to date, Publish · n (mint) and Failed · Retry (danger stroke);
	// Publishing… with its time, and a member's Ask to publish / Asked to publish as the icon alone,
	// are the desktop's states (G1). A viewer's pill says the state and does nothing. The twin
	// `mobile/ProjectNav` keeps the gallery's two states; this one is the workspace's.
	type Props = {
		project: string;
		status: string;
		live?: boolean;
		pill: PhonePill;
		onback: () => void;
		onmenu: () => void;
	};
	let { project, status, live = false, pill, onback, onmenu }: Props = $props();
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
	<span aria-live="polite" class="contents">
		{#if pill.kind === 'idle'}
			<span class="inline-flex h-9 shrink-0 items-center gap-1.5 rounded-2xl border border-border-strong px-3.25 text-ui whitespace-nowrap text-text-2">
				<span class="inline-flex text-text-3"><Icon icon={Check} size={14} /></span>{m.ws_up_to_date()}
			</span>
		{:else if pill.kind === 'ready'}
			<button type="button" class="inline-flex h-9 shrink-0 cursor-pointer items-center gap-1.5 rounded-2xl border border-accent bg-accent px-3.25 text-ui font-medium whitespace-nowrap text-accent-ink outline-offset-1 pointer-coarse:min-h-11" aria-haspopup="dialog" onclick={pill.onpublish}>
				<Icon icon={Rocket} size={14} />{m.pb_publish_count({ count: pill.count })}
			</button>
		{:else if pill.kind === 'publishing'}
			<span class="inline-flex h-9 shrink-0 items-center gap-1.5 rounded-2xl border border-border-strong px-3.25 text-ui whitespace-nowrap text-text-2">
				<span class="inline-flex animate-spin text-text-3 motion-reduce:animate-none"><Icon icon={LoaderCircle} size={14} /></span>{m.ws_publishing({ elapsed: pill.elapsed })}
			</span>
		{:else if pill.kind === 'failed'}
			{#if pill.onretry}
				<button type="button" class="inline-flex h-9 shrink-0 cursor-pointer items-center gap-1.5 rounded-2xl border border-danger px-3.25 text-ui whitespace-nowrap text-danger outline-offset-1 pointer-coarse:min-h-11" aria-haspopup="dialog" onclick={pill.onretry}>
					<Icon icon={CircleAlert} size={14} />{m.pb_failed_retry()}
				</button>
			{:else}
				<span class="inline-flex h-9 shrink-0 items-center gap-1.5 rounded-2xl border border-danger px-3.25 text-ui whitespace-nowrap text-danger"><Icon icon={CircleAlert} size={14} />{m.pb_failed()}</span>
			{/if}
		{:else if pill.kind === 'ask'}
			<button type="button" aria-label={m.publish_ask()} disabled={pill.busy} class="inline-flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-full text-text outline-offset-1 disabled:cursor-default disabled:text-text-3" onclick={pill.onask}>
				<Icon icon={Rocket} size={20} />
			</button>
		{:else if pill.kind === 'asked'}
			<button type="button" aria-label={m.publish_asked()} aria-haspopup="dialog" class="inline-flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-full text-text outline-offset-1" onclick={pill.onopen}>
				<Icon icon={Hourglass} size={20} />
			</button>
		{/if}
	</span>
	<button type="button" aria-label={m.project_options()} aria-haspopup="dialog" class="inline-flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-full text-text outline-offset-1" onclick={onmenu}>
		<Icon icon={Ellipsis} size={22} />
	</button>
</header>
