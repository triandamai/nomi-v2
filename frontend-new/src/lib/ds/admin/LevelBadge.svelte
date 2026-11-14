<script lang="ts">
	// Admin → Level badge: how much of an area someone may do (view, write, manage), in small mono
	// capitals like Tag, written in sentence case. Neutral at every level, a step stronger each time:
	// access is a fact, not good or bad news, so it takes no status colour.
	type Props = { level: 'view' | 'write' | 'manage'; label: string };
	let { level, label }: Props = $props();
</script>

<span
	data-level={level}
	class={[
		'inline-flex shrink-0 items-center rounded-xs border px-1.25 py-px font-mono text-label tracking-label uppercase',
		level === 'view' && 'border-border text-text-3',
		level === 'write' && 'border-border-strong text-text-2',
		level === 'manage' && 'border-border-strong bg-surface-2 text-text',
	]}>{label}</span
>
