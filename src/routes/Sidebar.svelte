<script lang="ts">
	import { page } from "$app/state"
	import ExecuteButton from "$lib/components/ExecuteButton.svelte"
	import LinkButton from "$lib/components/LinkButton.svelte"
	import { Tooltip } from "@skeletonlabs/skeleton-svelte"
	import type { Session } from "@supabase/supabase-js"
	import { invoke } from "@tauri-apps/api/core"
	import { fetch } from "@tauri-apps/plugin-http"
	import { revealItemInDir } from "@tauri-apps/plugin-opener"

	const session: Session = $derived(page.data.session)
	const path: string = $derived(page.data.simbaPath + "\\Plugins")

	let args = $derived(["", "latest", "latest", "", "", ""])
	let settingsBtn = $derived(page.url.pathname == "/settings" ? "/scripts" : "/settings")

	async function execute() {
		try {
			const response = await fetch("https://api.waspscripts.dev/session", {
				method: "GET",
				headers: {
					Authorization: "Bearer " + session.access_token,
					RefreshToken: session.refresh_token,
					"Content-Type": "application/json"
				}
			})
			const data = await response.json()
			args[5] = data.refresh_token
		} catch (err) {
			console.error(err)
		}

		await invoke("run_executable", { exe: "devsimba", args })
	}

	let openSimbaState = $state(false)
	let openFolderState = $state(false)
</script>

<div class="flex h-full flex-col justify-end gap-1 border-l border-surface-500 px-1">
	<Tooltip
		open={openSimbaState}
		onOpenChange={(e) => (openSimbaState = e.open)}
		positioning={{ placement: "top" }}
		triggerBase="underline"
		contentBase="card preset-filled p-4"
		openDelay={1000}
		arrow
	>
		{#snippet trigger()}
			<button class="btn preset-filled-surface-500 *:pointer-events-none" onclick={execute}>
				🦁
			</button>
		{/snippet}
		{#snippet content()}
			Update and Run Dev Environment
		{/snippet}
	</Tooltip>
	<Tooltip
		open={openFolderState}
		onOpenChange={(e) => (openFolderState = e.open)}
		positioning={{ placement: "top" }}
		triggerBase="underline"
		contentBase="card preset-filled p-4"
		openDelay={1000}
		arrow
	>
		{#snippet trigger()}
			<button
				class="btn preset-filled-surface-500 *:pointer-events-none"
				onclick={async () => await revealItemInDir(path)}
			>
				📁
			</button>
		{/snippet}
		{#snippet content()}
			Open Simba Folder
		{/snippet}
	</Tooltip>
	<ExecuteButton icon="💻" label="RuneLite" exe="runelite" args={[]} />
	<ExecuteButton icon="🚀" label="OSClient" exe="osclient" args={[]} />
	<LinkButton icon="⚙️" label="Settings" href={settingsBtn} />
</div>
