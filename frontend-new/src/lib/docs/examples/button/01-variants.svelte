<!-- Variants: one primary per view; inverse for the main action inside a panel, so mint stays rare. -->
<script lang="ts">
	import { ArrowRight, ArrowUpRight, Trash2, Users } from '@lucide/svelte';
	import Button from '$lib/ds/actions/Button.svelte';
</script>

<Button label="Start building" iconEnd={ArrowRight} />
<Button label="Open workspace" variant="inverse" iconEnd={ArrowRight} />
<Button label="Add credits on DeepSeek" variant="secondary" iconEnd={ArrowUpRight} />
<Button label="Team and invites" variant="ghost" iconStart={Users} />
<Button label="Delete project" variant="danger" iconStart={Trash2} />
