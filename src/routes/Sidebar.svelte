<script lang="ts">
	import { page } from "$app/state"
	import { devModeStore, devPathStore, devUpdatesStore } from "$lib/store"
	import { supabase } from "$lib/supabase"
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

	let runningBtn = $derived(page.url.pathname.includes("/running") ? "/scripts" : "/running")

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
</script>

<div class="flex h-full flex-col justify-between border-l border-surface-500">
	<Tooltip positioning={{ placement: "top" }} openDelay={700}>
		<Tooltip.Trigger>
			<a
				href={runningBtn}
				class="btn preset-filled-surface-500 *:pointer-events-none"
				data-sveltekit-preload-data="false"
			>
				🎮
			</a>
		</Tooltip.Trigger>
		<Portal>
			<Tooltip.Positioner>
				<Tooltip.Content class="card preset-filled p-4">Running</Tooltip.Content>
			</Tooltip.Positioner>
		</Portal>
	</Tooltip>
	<div class="flex h-full flex-col justify-end gap-1 px-1">
		{#if $devModeStore}
			<Tooltip positioning={{ placement: "top" }} openDelay={700}>
				<Tooltip.Trigger>
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
				<Tooltip.Trigger>
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
			<Tooltip.Trigger>
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
			<Tooltip.Trigger>
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

		<Tooltip positioning={{ placement: "top" }} openDelay={700}>
			<Tooltip.Trigger>
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
</div>
