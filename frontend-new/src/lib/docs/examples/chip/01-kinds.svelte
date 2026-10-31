<!-- Kinds: model, option and agent open a menu; provision only names what a project has. -->
<script lang="ts">
	import { GitBranch, Lightbulb } from '@lucide/svelte';
	import Chip from '$lib/ds/inputs/Chip.svelte';
</script>

<Chip kind="model" label="deepseek-flash" onclick={() => {}} />
<Chip kind="option" label="Default" icon={Lightbulb} onclick={() => {}} />
<Chip kind="agent" label="Nomi" agent="nomi" onclick={() => {}} />
<Chip kind="provision" label="git repo" icon={GitBranch} />
