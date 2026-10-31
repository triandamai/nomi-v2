<!-- Peers and sizes: teammates in their colour, you neutral; a ring for who is here now. -->
<script lang="ts">
	import PeerAvatar from '$lib/ds/build/PeerAvatar.svelte';
</script>

<PeerAvatar initials="MP" />
<PeerAvatar initials="SW" peer={1} />
<PeerAvatar initials="SW" peer={1} ring />
<PeerAvatar initials="SW" peer={1} size={20} />
