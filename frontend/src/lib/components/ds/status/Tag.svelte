<script lang="ts">
	// A go / no-go / standby verdict. Written in sentence case ("No-go"), shown in the meta face.
	let { kind, label }: { kind: 'go' | 'no-go' | 'neutral'; label: string } = $props();
</script>

<span class="tag tag--{kind}">{label}</span>

<style>
	.tag {
		display: inline-flex;
		align-items: center;
		height: 24px;
		padding: 0 8px;
		border-radius: var(--md-sys-shape-corner-small);
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
		font-weight: 500;
		letter-spacing: 0.06em;
		text-transform: uppercase;
	}
	.tag--go {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.tag--no-go {
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}
	.tag--neutral {
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
