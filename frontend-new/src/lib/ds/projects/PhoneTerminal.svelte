<script lang="ts">
	import { CircleAlert } from '@lucide/svelte';
	import PhoneButton from '../mobile/PhoneButton.svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile Build — Code, terminal & phone preview (K7RZh1, D11w0) → Terminal on a phone: the
	// log in one 14-round card, 14 in, lines 6 apart at 12 / 1.4 in mono, wrapped (never sideways),
	// coloured by kind (command, out, ok in accent, fail and error in danger, warn in warning). A
	// failure sums up in the fix bar at the foot ([14,16,34,16] on surface, a line on top): what
	// went wrong, what it means, and the fix as the full-width button.
	type Line = { kind: 'command' | 'out' | 'ok' | 'fail' | 'error' | 'warn' | 'blank'; text?: string };
	type Props = { label: string; lines: Line[]; failure?: { title: string; detail?: string; fix: string; onfix: () => void } };
	let { label, lines, failure }: Props = $props();
</script>

<div class="flex min-h-0 w-full flex-1 flex-col">
	<div class="min-h-0 flex-1 overflow-y-auto px-4 pt-4 pb-5">
		<div role="log" aria-label={label} class="flex flex-col gap-1.5 rounded-xl p-3.5 font-mono text-caption leading-cozy outline -outline-offset-1 outline-border">
			{#each lines as line, index (index)}
				<p
					class={[
						'break-all whitespace-pre-wrap',
						line.kind === 'command' && 'text-text',
						line.kind === 'out' && 'text-text-2',
						line.kind === 'ok' && 'text-accent',
						(line.kind === 'fail' || line.kind === 'error') && 'text-danger',
						line.kind === 'warn' && 'text-warning',
					]}
				>
					{line.text ?? ''}
				</p>
			{/each}
		</div>
	</div>
	{#if failure}
		<footer role="alert" class="flex flex-col gap-2.5 border-t border-border bg-surface px-4 pt-3.25 pb-safe">
			<div class="flex items-center gap-2.5">
				<span class="inline-flex shrink-0 text-danger"><Icon icon={CircleAlert} size={18} /></span>
				<span class="flex min-w-0 flex-1 flex-col gap-0.5">
					<span class="text-body font-medium text-text">{failure.title}</span>
					{#if failure.detail}<span class="text-ui break-words text-text-2">{failure.detail}</span>{/if}
				</span>
			</div>
			<div class="pb-4"><PhoneButton label={failure.fix} onclick={failure.onfix} /></div>
		</footer>
	{/if}
</div>
