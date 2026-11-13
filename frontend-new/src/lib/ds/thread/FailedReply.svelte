<script lang="ts">
	import type { ComponentProps } from 'svelte';
	import ErrorCallout from '../conversation/ErrorCallout.svelte';

	// Flow / Chat — Errors & recovery (TpuRc): a turn that failed, in the reply's place. First what
	// happened in plain words (the backend's own sentence, body 15 at 1.6 in text-2), then the
	// Error Callout naming the failure with the next moves, the main one first. The callout carries
	// only what was reported: the backend sends no status code, request id or provider reason with
	// a failed turn yet (BACKEND_TODO.md), so none are drawn.
	type Props = { text: string; title: string; details?: ComponentProps<typeof ErrorCallout>['details']; actions?: ComponentProps<typeof ErrorCallout>['actions'] };
	let { text, title, details = [], actions = [] }: Props = $props();
</script>

<div class="flex w-full flex-col gap-4">
	<p class="text-body leading-relaxed whitespace-pre-wrap text-text-2">{text}</p>
	<ErrorCallout {title} {details} {actions} />
</div>
