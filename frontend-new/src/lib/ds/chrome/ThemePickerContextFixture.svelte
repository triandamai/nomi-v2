<script lang="ts">
	import type { Theme } from '$lib/theme';
	import ThemePickerButton from './ThemePickerButton.svelte';
	import { setThemeContext } from './themeContext';

	type Props = {
		theme: Theme;
		changeTheme: (theme: Theme) => void;
	};
	let { theme, changeTheme }: Props = $props();

	setThemeContext({
		get theme() {
			return theme;
		},
		changeTheme(next) {
			changeTheme(next);
		},
	});
</script>

<ThemePickerButton />
