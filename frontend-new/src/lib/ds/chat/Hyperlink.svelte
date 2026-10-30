<script lang="ts">
	// Chat responses → Hyperlink: a link inside an answer, a step brighter than the sentence around
	// it and underlined faintly, so it isn't told apart by colour alone. Takes the sentence's size.
	type Props = { href: string; label: string; external?: boolean };
	let { href, label, external = false }: Props = $props();
</script>

<a
	{href}
	target={external ? '_blank' : undefined}
	rel={external ? 'noopener noreferrer' : undefined}
	class="rounded-xs text-text underline decoration-border-strong underline-offset-2 outline-offset-1 hover:decoration-text-3">{label}</a
>
