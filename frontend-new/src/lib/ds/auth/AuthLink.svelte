<script lang="ts">
	// Auth / Sign in or create → "Use a password instead", Enter code → Alt: the quiet ways off the
	// form, in the caption size: text-2, or text-3 when quiet. Underlined on hover; a link when it
	// goes somewhere, a button when it does something on this page. 44 high on a phone.
	type Props = {
		label: string;
		href?: string;
		quiet?: boolean;
		type?: 'button' | 'submit';
		onclick?: (event: MouseEvent) => void;
	};
	let { label, href, quiet = false, type = 'button', onclick }: Props = $props();
</script>

{#if href}
	<a
		{href}
		class={[
			'inline-flex items-center rounded-xs text-caption outline-offset-2 hover:underline hover:underline-offset-2 max-sm:min-h-11',
			quiet ? 'text-text-3 hover:text-text-2' : 'text-text-2 hover:text-text',
		]}>{label}</a
	>
{:else}
	<button
		{type}
		{onclick}
		class={[
			'inline-flex cursor-pointer items-center rounded-xs text-caption outline-offset-2 hover:underline hover:underline-offset-2 max-sm:min-h-11',
			quiet ? 'text-text-3 hover:text-text-2' : 'text-text-2 hover:text-text',
		]}>{label}</button
	>
{/if}
