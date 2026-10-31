<!-- In a button: it goes wherever an icon does. -->
<script lang="ts">
	import Button from '$lib/ds/actions/Button.svelte';
	import GithubMark from '$lib/ds/foundations/GithubMark.svelte';
</script>

<Button label="Push to GitHub" variant="secondary" iconEnd={GithubMark} />
