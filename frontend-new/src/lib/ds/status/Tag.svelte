<script lang="ts">
	// Design System / Nomi → Status: a go / no-go / standby verdict in small mono capitals.
	// Written in sentence case ("No-go"); shown in capitals.
	let { kind, label }: { kind: 'go' | 'no-go' | 'neutral'; label: string } = $props();
</script>

<span
	class={[
		'inline-flex items-center rounded-xs border px-1.5 py-0.5 font-mono text-label tracking-label uppercase',
		kind === 'no-go' && 'border-danger-border bg-danger-bg text-danger',
		kind === 'go' && 'border-accent-bg bg-accent-bg text-accent',
		kind === 'neutral' && 'border-border-strong bg-surface-2 text-text-2',
	]}>{label}</span
>
