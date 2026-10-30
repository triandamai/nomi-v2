<script lang="ts">
	// Chat responses → Hyperlink: a link inside an answer, a step brighter than the sentence around
	// it (as the pen draws it), underlined on hover. Takes the sentence's size.
	type Props = { href: string; label: string; external?: boolean };
	let { href, label, external = false }: Props = $props();
</script>

<a
	{href}
	target={external ? '_blank' : undefined}
	rel={external ? 'noopener noreferrer' : undefined}
	class="rounded-xs text-text outline-offset-1 hover:underline hover:decoration-text-3 hover:underline-offset-2">{label}</a
>
