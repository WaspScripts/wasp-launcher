<script lang="ts">
	import { page } from "$app/state"
	import ExecuteButton from "$lib/components/ExecuteButton.svelte"
	import LinkButton from "$lib/components/LinkButton.svelte"
	import { refreshSession } from "$lib/supabase"
	import { Tooltip } from "@skeletonlabs/skeleton-svelte"
	import type { Session } from "@supabase/supabase-js"
	import { invoke } from "@tauri-apps/api/core"

	const session: Session = $derived(page.data.session)

	const args = $derived([
		"",
		"latest",
		"latest",
		"",
		"",
		session.access_token,
		session.refresh_token
	])

	let settingsBtn = $derived(page.url.pathname == "/settings" ? "/scripts" : "/settings")

	async function execute() {
		const res = await invoke("run_executable", { exe: "devsimba", args })
		console.log(res)
	}
	let openState = $state(false)
</script>

<div class="flex h-full flex-col justify-end gap-1 border-l-1 border-surface-500 px-1">
	<Tooltip
		open={openState}
		onOpenChange={(e) => (openState = e.open)}
		positioning={{ placement: "top" }}
		triggerBase="underline"
		contentBase="card preset-filled p-4"
		openDelay={1000}
		arrow
	>
		{#snippet trigger()}
			<button class="btn preset-filled-surface-500 [&>*]:pointer-events-none" onclick={execute}>
				🦁
			</button>
		{/snippet}
		{#snippet content()}
			Open Simba
		{/snippet}
	</Tooltip>
	<ExecuteButton icon="💻" label="RuneLite" exe="runelite" args={[]} />
	<ExecuteButton icon="🚀" label="OSClient" exe="osclient" args={[]} />
	<LinkButton icon="⚙️" label="Settings" href={settingsBtn} />
</div>
