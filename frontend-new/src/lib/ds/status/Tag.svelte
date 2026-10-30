<script lang="ts">
	// Design System / Nomi → Status: a go / no-go / standby verdict in small mono capitals.
	// Written in sentence case ("No-go"); shown in capitals. The stroke sits inside the pen's
	// 2px / 6px padding, so 18 tall.
	let { kind, label }: { kind: 'go' | 'no-go' | 'neutral'; label: string } = $props();
</script>

<span
	class={[
		'inline-flex items-center rounded-xs border px-1.25 py-px font-mono text-label tracking-label uppercase',
		kind === 'no-go' && 'border-danger-border bg-danger-bg text-danger',
		kind === 'go' && 'border-accent-bg bg-accent-bg text-accent',
		kind === 'neutral' && 'border-border-strong bg-surface-2 text-text-2',
	]}>{label}</span
>
