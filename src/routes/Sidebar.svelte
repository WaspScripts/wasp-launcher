<script lang="ts">
	import { page } from "$app/state"
	import ExecuteButton from "$lib/components/ExecuteButton.svelte"
	import { devModeStore, devPathStore, devUpdatesStore } from "$lib/store"
	import { Tooltip, Portal } from "@skeletonlabs/skeleton-svelte"
	import type { Session } from "@supabase/supabase-js"
	import { invoke } from "@tauri-apps/api/core"
	import { fetch } from "@tauri-apps/plugin-http"
	import { revealItemInDir } from "@tauri-apps/plugin-opener"

	const session: Session = $derived(page.data.session)
	const path: string = $derived(page.data.simbaPath + "//Plugins")

	let settingsBtn = $derived(
		page.url.pathname.includes("/settings") ? "/scripts" : "/settings/general"
	)

	async function execute(exe: string, wasplib: string) {
		let refresh_token = ""
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
			refresh_token = data.refresh_token
		} catch (err) {
			console.error(err)
		}

		await invoke("run_executable", { exe, args: ["", "latest", wasplib, "", "", refresh_token] })
	}
</script>

<div class="flex h-full flex-col justify-end gap-1 border-l border-surface-500 px-1">
	{#if $devModeStore}
		<Tooltip positioning={{ placement: "top" }} openDelay={700}>
			<Tooltip.Trigger class="underline">
				<button
					class="btn preset-filled-surface-500 *:pointer-events-none"
					onclick={() => execute("devsimba", $devUpdatesStore ? "latest" : "none")}
				>
					🧪
				</button>
			</Tooltip.Trigger>
			<Portal>
				<Tooltip.Positioner>
					<Tooltip.Content class="card preset-filled p-4">Run Development Simba</Tooltip.Content>
				</Tooltip.Positioner>
			</Portal>
		</Tooltip>

		<Tooltip positioning={{ placement: "top" }} openDelay={700}>
			<Tooltip.Trigger class="underline">
				<button
					class="btn preset-filled-surface-500 *:pointer-events-none"
					onclick={async () => await revealItemInDir($devPathStore + "//Plugins")}
				>
					💻
				</button>
			</Tooltip.Trigger>
			<Portal>
				<Tooltip.Positioner>
					<Tooltip.Content class="card preset-filled p-4"
						>Open Development Directory</Tooltip.Content
					>
				</Tooltip.Positioner>
			</Portal>
		</Tooltip>
	{/if}
	<Tooltip positioning={{ placement: "top" }} openDelay={700}>
		<Tooltip.Trigger class="underline">
			<button
				class="btn preset-filled-surface-500 *:pointer-events-none"
				onclick={() => execute("simba", "latest")}
			>
				🦁
			</button>
		</Tooltip.Trigger>
		<Portal>
			<Tooltip.Positioner>
				<Tooltip.Content class="card preset-filled p-4">Update and Run Simba</Tooltip.Content>
			</Tooltip.Positioner>
		</Portal>
	</Tooltip>

	<Tooltip positioning={{ placement: "top" }} openDelay={700}>
		<Tooltip.Trigger class="underline">
			<button
				class="btn preset-filled-surface-500 *:pointer-events-none"
				onclick={async () => await revealItemInDir(path)}
			>
				📁
			</button>
		</Tooltip.Trigger>
		<Portal>
			<Tooltip.Positioner>
				<Tooltip.Content class="card preset-filled p-4">Open Simba Directory</Tooltip.Content>
			</Tooltip.Positioner>
		</Portal>
	</Tooltip>

	<ExecuteButton icon="⚡" label="RuneLite" exe="runelite" args={[]} />
	<ExecuteButton icon="🔥" label="OSClient" exe="osclient" args={[]} />

	<Tooltip positioning={{ placement: "top" }} openDelay={700}>
		<Tooltip.Trigger class="underline">
			<a
				href={settingsBtn}
				class="btn preset-filled-surface-500 *:pointer-events-none"
				data-sveltekit-preload-data="false"
			>
				⚙️
			</a>
		</Tooltip.Trigger>
		<Portal>
			<Tooltip.Positioner>
				<Tooltip.Content class="card preset-filled p-4">Settings</Tooltip.Content>
			</Tooltip.Positioner>
		</Portal>
	</Tooltip>
</div>
