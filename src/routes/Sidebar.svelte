<script lang="ts">
	import { page } from "$app/state"
	import ExecuteButton from "$lib/components/ExecuteButton.svelte"
	import LinkButton from "$lib/components/LinkButton.svelte"
	import type { Session } from "@supabase/supabase-js"

	const session: Session = $derived(page.data.session)

	const args = $derived(["", "latest", "latest", "", "", session.refresh_token])

	let settingsBtn = $derived(page.url.pathname == "/settings" ? "/scripts" : "/settings")
</script>

<div class="flex h-full flex-col justify-end gap-1 border-l-1 border-surface-500 px-1">
	<ExecuteButton icon="🦁" label="Open Simba" exe="devsimba" {args} />
	<ExecuteButton icon="💻" label="RuneLite" exe="runelite" args={[]} />
	<ExecuteButton icon="🚀" label="OSClient" exe="osclient" args={[]} />
	<LinkButton icon="⚙️" label="Settings" href={settingsBtn} />
</div>
