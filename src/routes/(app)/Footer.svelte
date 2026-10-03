<script lang="ts">
	import { Popover, Portal, Tooltip } from "@skeletonlabs/skeleton-svelte"
	import { invoke } from "@tauri-apps/api/core"
	import type { ScriptEx } from "$lib/types/collection"
	import { page } from "$app/state"
	import type { Session, SupabaseClient } from "@supabase/supabase-js"
	import type { Database } from "$lib/types/supabase"
	import { fetch } from "@tauri-apps/plugin-http"
	import { RefreshCw, Share2, SquaresSubtract } from "@lucide/svelte"
	import { channelManager } from "$lib/communication.svelte"
	import { goto } from "$app/navigation"
	import { assetsURLStore } from "$lib/store"
	import { MediaQuery } from "svelte/reactivity"
	import type { Component } from "svelte"

	let data = $props()
	let script: ScriptEx = $derived(data.script)
	const supabase: SupabaseClient<Database> = $derived(page.data.supabase)
	const session: Session = $derived(page.data.session)

	function pad(n: number, size: number) {
		let s = n + ""
		while (s.length < size) s = "0" + s
		return s
	}

	async function saveBlobToFile(blob: Blob, path: string, filename: string) {
		const arrayBuffer = await blob.arrayBuffer()
		const data = Array.from(new Uint8Array(arrayBuffer))

		await invoke("save_blob", { path, filename, data })
	}

	async function getVersions(id: string) {
		const { data, error: err } = await supabase
			.schema("scripts")
			.from("versions")
			.select("revision, simba, wasplib, files")
			.eq("id", id)
			.order("revision", { ascending: false })

		if (err) {
			console.error(err)
			return []
		}
		return data
	}

	const versionsPromise = $derived(getVersions(script.id))
	let revision = $state(0)

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

	async function execute() {
		const versions = await versionsPromise
		const version = versions[revision]

		const scriptName = script.url + "-rev-" + version.revision
		const mainFile = scriptName + "/" + scriptName + ".simba"

		const downloads = version.files.map(async (filename) => {
			const filepath = script.id + "/" + pad(version.revision, 9) + "/" + filename
			console.log("Downloading file:", filepath)
			const { data, error: err } = await supabase.storage.from("scripts").download(filepath)

			if (err) {
				console.error(err)
				return false
			}

			const file = filename == "script.simba" ? scriptName + ".simba" : filename
			await saveBlobToFile(data, scriptName, file)
			return true
		})

		const [refreshToken, saved] = await Promise.all([getNewSessionToken(), Promise.all(downloads)])
		if (saved.includes(false)) return

		const args = [
			mainFile,
			version.simba,
			version.wasplib,
			script.id,
			script.protected.revision.toString(),
			refreshToken,
			$assetsURLStore
		]

		const channel = await channelManager.createChannel(script.title)
		const result = await invoke("run_script", { args, channel })
		console.log("run_script: ", result)
		return channel.id
	}

	let client = $state(-1)

	let lazyGithub = import("./Footer/GitHub.svelte")
	let lazyDiscord = import("./Footer/Discord.svelte")
	let lazyYouTube = import("./Footer/YouTube.svelte")

	// tailwind's `md` and `lg` breakpoints
	const md = new MediaQuery("min-width: 48rem")
	const lg = new MediaQuery("min-width: 64rem")

	interface ClientWindow {
		pid: number
		hwnd: number
		name: string
	}
	let clientsPromise = $state(invoke("list_clients") as Promise<ClientWindow[]>)
</script>

<footer
	class="sticky bottom-0 flex justify-between gap-2 bg-surface-200/30 p-2 text-base font-semibold backdrop-blur-md md:p-4 dark:bg-surface-800/30"
>
	{#snippet social(lazy: Promise<{ default: Component }>, label: string)}
		{#await lazy then { default: Icon }}
			<Tooltip positioning={{ placement: "top" }} openDelay={1000}>
				<Tooltip.Trigger>
					<Icon />
				</Tooltip.Trigger>
				<Portal>
					<Tooltip.Positioner>
						<Tooltip.Content class="card preset-filled p-4">{label}</Tooltip.Content>
					</Tooltip.Positioner>
				</Portal>
			</Tooltip>
		{/await}
	{/snippet}

	{#snippet socials()}
		{@render social(lazyGithub, "Source code")}
		{@render social(lazyDiscord, "Join the Discord community!")}
		{@render social(lazyYouTube, "YouTube channel")}
	{/snippet}

	{#if md.current}
		<div class="flex gap-2">
			{@render socials()}
		</div>
	{:else}
		<Popover positioning={{ placement: "top-start" }}>
			<Popover.Trigger class="my-auto btn h-8 hover:preset-tonal" aria-label="Community links">
				<Share2 size={20} />
			</Popover.Trigger>
			<Portal>
				<Popover.Positioner>
					<Popover.Content class="flex gap-2 card bg-surface-100-900 p-2 shadow-xl">
						{@render socials()}
					</Popover.Content>
				</Popover.Positioner>
			</Portal>
		</Popover>
	{/if}

	{#if script}
		<div class="flex min-w-0 gap-2">
			{#if script.access}
				<div class="input-group h-8 grid-cols-[auto_1fr_auto]">
					<button
						class="group ig-cell gap-2 hover:preset-tonal"
						title={lg.current ? undefined : "Refresh clients"}
						onclick={async () => {
							client = -1
							clientsPromise = invoke("list_clients") as Promise<ClientWindow[]>
							await invoke("set_client", {})
						}}
					>
						<span
							class="max-w-0 overflow-hidden whitespace-nowrap opacity-0 duration-300 lg:group-hover:max-w-32 lg:group-hover:opacity-100"
						>
							Refresh clients
						</span>
						<RefreshCw size={16} class="duration-500 group-hover:rotate-180" />
					</button>

					<button
						class="group ig-cell gap-2 enabled:hover:preset-tonal"
						disabled={client < 0}
						title={lg.current ? undefined : "Show client"}
						onclick={async () => {
							await invoke("show_client")
						}}
					>
						<span
							class="max-w-0 overflow-hidden whitespace-nowrap opacity-0 duration-300
							lg:group-enabled:group-hover:max-w-32 lg:group-enabled:group-hover:opacity-100"
						>
							Show client
						</span>
						<SquaresSubtract size={16} />
					</button>

					<select
						id="client"
						class="select ig-select w-32 rounded-l-none hover:preset-tonal md:w-40 lg:w-48"
						bind:value={client}
						onchange={async () => {
							const clients = await clientsPromise
							await invoke("set_client", { client: clients[client] })
						}}
					>
						<option value={-1} disabled selected>{lg.current ? "Select a client" : "Client"}</option
						>
						{#await clientsPromise then clients}
							{#each clients as clnt, idx}
								<option value={idx}>
									{clnt.name}
								</option>
							{/each}
						{/await}
					</select>
				</div>

				<select
					id="revision"
					class="select w-28 hover:preset-tonal md:w-36 lg:w-44"
					bind:value={revision}
				>
					{#await versionsPromise then versions}
						{#each versions as version, idx}
							<option value={idx}>{lg.current ? "Revision" : "Rev."} {version.revision}</option>
						{/each}
					{/await}
				</select>

				<Tooltip positioning={{ placement: "top" }} openDelay={1000}>
					<Tooltip.Trigger>
						<button
							class="hover:preset-filled-primary-800 btn preset-filled-primary-500"
							onclick={async () => {
								const id = await execute()
								await goto("/running/" + id)
							}}
							disabled={client < 0}
						>
							Run
						</button>
					</Tooltip.Trigger>
					<Portal>
						<Tooltip.Positioner>
							<Tooltip.Content class="card preset-filled p-4">Open in Simba</Tooltip.Content>
						</Tooltip.Positioner>
					</Portal>
				</Tooltip>
			{:else}
				<Tooltip positioning={{ placement: "top" }} openDelay={1000}>
					<Tooltip.Trigger class="m-auto">
						<a
							class="btn preset-filled-primary-500 hover:preset-tonal"
							href="https://waspscripts.dev/scripts/{script.id}"
							target="_blank"
						>
							Buy
						</a>
					</Tooltip.Trigger>
					<Portal>
						<Tooltip.Positioner>
							<Tooltip.Content class="card preset-filled p-4">Buy Script</Tooltip.Content>
						</Tooltip.Positioner>
					</Portal>
				</Tooltip>
			{/if}
		</div>
	{/if}
</footer>
