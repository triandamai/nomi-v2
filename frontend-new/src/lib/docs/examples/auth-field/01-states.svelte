<!-- States: focused as the pen draws it, with a helper, with an error, and a value that came with the link. -->
<script lang="ts">
	import { Building2, LockKeyhole, Mail, Ticket } from '@lucide/svelte';
	import AuthField from '$lib/ds/auth/AuthField.svelte';
</script>

<div class="grid w-full max-w-200 gap-6 sm:grid-cols-2">
	<AuthField label="Email" type="email" icon={Mail} placeholder="you@example.com" data-focus />
	<AuthField label="Workspace name" icon={Building2} placeholder="Family budget" helper="Where your chats and projects live. You can invite people later." />
	<AuthField label="Password" type="password" icon={LockKeyhole} value="short" error="Use at least 8 characters." />
	<AuthField label="Invite code" icon={Ticket} value="Xk29fQpL0aZ7" readonly mono helper="From your invite: your account joins that workspace." />
</div>
