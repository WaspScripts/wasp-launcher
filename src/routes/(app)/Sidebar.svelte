<script lang="ts">
	import { page } from "$app/state"
	import { devModeStore, devPathStore, devUpdatesStore } from "$lib/store"
	import { supabase } from "$lib/supabase"
	import { PanelLeftOpen } from "@lucide/svelte"
	import { Tooltip, Portal } from "@skeletonlabs/skeleton-svelte"
	import type { Session } from "@supabase/supabase-js"
	import { invoke } from "@tauri-apps/api/core"
	import { fetch } from "@tauri-apps/plugin-http"
	import { revealItemInDir } from "@tauri-apps/plugin-opener"

	const { settings, sidebar } = $derived(page.data)
	const session: Session = $derived(page.data.session)
	const path: string = $derived(page.data.simbaPath + "//Plugins")

	let settingsBtn = $derived(
		page.url.pathname.startsWith("/settings") ? "/scripts" : "/settings/general"
	)

	let runningBtn = $derived(page.url.pathname.startsWith("/running") ? "/scripts" : "/running")

	async function getNewSessionToken() {
		let result = ""
		try {
			const response = await fetch("https://api.waspscripts.dev/session", {
				method: "GET",
				headers: {
					authorization: "Bearer " + session.access_token,
					refreshtoken: session.refresh_token,
					"Content-Type": "application/json"
				}
			})
			const data = await response.json()
			result = data.refresh_token
		} catch (err) {
			console.error(err)
		}

		return result
	}

	async function execute(exe: string, wasplib: string) {
		const promises = await Promise.all([
			getNewSessionToken(),
			supabase
				.schema("scripts")
				.from("wasplib")
				.select("simba")
				.order("created_at", { ascending: false })
				.limit(1)
				.single()
		])

		let refresh_token = promises[0]

		const { data, error: versionErr } = promises[1]

		if (versionErr) {
			console.error(versionErr)
			return
		}

		await invoke("run_executable", { exe, args: ["", data.simba, wasplib, "", "", refresh_token] })
	}

	// svelte-ignore state_referenced_locally
	let currentSidebar = $state(sidebar) as boolean

	async function toggleSidebar() {
		currentSidebar = !currentSidebar
		document.documentElement.classList.toggle("sidebar")
		await settings.set("sidebar", currentSidebar)
	}
</script>

<div class="flex h-full flex-col justify-between gap-1 border-l border-surface-500 px-1">
	<Tooltip positioning={{ placement: "top" }} openDelay={700}>
		<Tooltip.Trigger
			onclick={toggleSidebar}
			class="btn h-9 w-full preset-filled-surface-500 text-xs *:pointer-events-none lg:text-sm"
		>
			<span class="duration-400" class:rotate-180={!currentSidebar}>
				<PanelLeftOpen size="20" />
			</span>
			{#if currentSidebar}
				Collapse
			{/if}
		</Tooltip.Trigger>
		<Portal>
			<Tooltip.Positioner>
				<Tooltip.Content class="card preset-filled p-4"
					>{#if currentSidebar}
						Collapse
					{:else}Expand{/if}</Tooltip.Content
				>
			</Tooltip.Positioner>
		</Portal>
	</Tooltip>

	<Tooltip positioning={{ placement: "top" }} openDelay={700}>
		<Tooltip.Trigger>
			<a
				href={runningBtn}
				class="btn h-9 w-full preset-filled-surface-500 text-xs *:pointer-events-none lg:text-sm"
				data-sveltekit-preload-data="false"
			>
				🎮
				{#if currentSidebar}
					Running
				{/if}
			</a>
		</Tooltip.Trigger>
		<Portal>
			<Tooltip.Positioner>
				<Tooltip.Content class="card preset-filled p-4">Show running scripts</Tooltip.Content>
			</Tooltip.Positioner>
		</Portal>
	</Tooltip>

	<div class="flex h-full flex-col justify-end gap-1 px-1">
		{#if $devModeStore}
			<Tooltip positioning={{ placement: "top" }} openDelay={700}>
				<Tooltip.Trigger>
					<button
						class="btn h-9 w-full preset-filled-surface-500 text-xs *:pointer-events-none lg:text-sm"
						onclick={() => execute("devsimba", $devUpdatesStore ? "latest" : "none")}
					>
						🧪 {#if currentSidebar}
							Dev Simba
						{/if}
					</button>
				</Tooltip.Trigger>
				<Portal>
					<Tooltip.Positioner>
						<Tooltip.Content class="card preset-filled p-4">Run Development Simba</Tooltip.Content>
					</Tooltip.Positioner>
				</Portal>
			</Tooltip>

			<Tooltip positioning={{ placement: "top" }} openDelay={700}>
				<Tooltip.Trigger>
					<button
						class="btn h-9 w-full preset-filled-surface-500 text-xs *:pointer-events-none lg:text-sm"
						onclick={async () => await revealItemInDir($devPathStore + "//Plugins")}
					>
						💻 {#if currentSidebar}
							Dev Folder
						{/if}
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
			<Tooltip.Trigger>
				<button
					class="btn h-9 w-full preset-filled-surface-500 text-xs *:pointer-events-none lg:text-sm"
					onclick={() => execute("simba", "latest")}
				>
					🦁
					{#if currentSidebar}
						Simba
					{/if}
				</button>
			</Tooltip.Trigger>
			<Portal>
				<Tooltip.Positioner>
					<Tooltip.Content class="card preset-filled p-4">Update and Run Simba</Tooltip.Content>
				</Tooltip.Positioner>
			</Portal>
		</Tooltip>

		<Tooltip positioning={{ placement: "top" }} openDelay={700}>
			<Tooltip.Trigger>
				<button
					class="btn h-9 w-full preset-filled-surface-500 text-xs *:pointer-events-none lg:text-sm"
					onclick={async () => await revealItemInDir(path)}
				>
					📁
					{#if currentSidebar}
						Simba Folder
					{/if}
				</button>
			</Tooltip.Trigger>
			<Portal>
				<Tooltip.Positioner>
					<Tooltip.Content class="card preset-filled p-4">Open Simba Directory</Tooltip.Content>
				</Tooltip.Positioner>
			</Portal>
		</Tooltip>

		<Tooltip positioning={{ placement: "top" }} openDelay={700}>
			<Tooltip.Trigger>
				<a
					href={settingsBtn}
					class="btn h-9 w-full preset-filled-surface-500 text-xs *:pointer-events-none lg:text-sm"
					data-sveltekit-preload-data="false"
				>
					⚙️ {#if currentSidebar}
						Settings
					{/if}
				</a>
			</Tooltip.Trigger>
			<Portal>
				<Tooltip.Positioner>
					<Tooltip.Content class="card preset-filled p-4">Settings</Tooltip.Content>
				</Tooltip.Positioner>
			</Portal>
		</Tooltip>
	</div>
</div>
