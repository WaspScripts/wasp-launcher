<script lang="ts">
	import { Tooltip } from "@skeletonlabs/skeleton-svelte"
	import { invoke } from "@tauri-apps/api/core"
	import Discord from "./Footer/Discord.svelte"
	import GitHub from "./Footer/GitHub.svelte"
	import YouTube from "./Footer/YouTube.svelte"
	import type { ScriptEx } from "$lib/types/collection"
	import { page } from "$app/state"
	import type { Session, SupabaseClient } from "@supabase/supabase-js"
	import type { Database } from "$lib/types/supabase"
	import { fetch } from "@tauri-apps/plugin-http"

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
	let selected = $state(0)

	async function getNewSessionToken() {
		let result = ""
		try {
			const response = await fetch("http://localhost:3000/session", {
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
		const version = versions[selected]

		let promises = []
		promises.push(getNewSessionToken())

		const scriptName = script.url + "-rev-" + version.revision
		const mainFile = scriptName + "/" + scriptName + ".simba"

		for (let i = 0; i < version.files.length; i++) {
			const filepath = script.id + "/" + pad(version.revision, 9) + "/" + version.files[i]
			console.log("Downloading file:", filepath)
			const { data, error: err } = await supabase.storage.from("scripts").download(filepath)

			if (err) {
				console.error(err)
				return
			}

			const file = version.files[i] == "script.simba" ? scriptName + ".simba" : version.files[i]
			promises.push(saveBlobToFile(data, scriptName, file))
		}

		const awaitedPromises = await Promise.all(promises)

		let refreshToken = awaitedPromises[0] as string
		console.log(refreshToken)

		const exe = "simba"
		const args = [
			mainFile,
			version.simba,
			version.wasplib,
			script.id,
			script.protected.revision.toString(),
			refreshToken
		]

		await invoke("run_executable", { exe, args })
	}
	let openState = $state(false)
</script>

<footer
	class="sticky bottom-0 flex justify-between bg-surface-200/30 text-base font-semibold backdrop-blur-md dark:bg-surface-800/30"
>
	<div class="mx-4 my-4 flex gap-2">
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
				<GitHub />
			{/snippet}
			{#snippet content()}Source code{/snippet}
		</Tooltip>

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
				<Discord />
			{/snippet}
			{#snippet content()}Join the Discord community!{/snippet}
		</Tooltip>

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
				<YouTube />
			{/snippet}
			{#snippet content()}YouTube channel{/snippet}
		</Tooltip>
	</div>

	{#if script}
		<div class="flex">
			{#if script.access}
				{#await versionsPromise}
					<select class="my-auto select"> Loading... </select>
				{:then versions}
					<select class="my-auto select" bind:value={selected}>
						{#each versions as version, idx}
							<option value={idx}>Revision {version.revision}</option>
						{/each}
					</select>
				{/await}

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
						<button class="mx-4 my-4 btn preset-filled-primary-500" onclick={execute}>Run</button>
					{/snippet}
					{#snippet content()}Open in Simba{/snippet}
				</Tooltip>
			{:else}
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
						<a
							class="mx-4 my-4 btn preset-filled-primary-500"
							href="https://waspscripts.dev/scripts/{script.id}"
							target="_blank"
						>
							Buy
						</a>
					{/snippet}
					{#snippet content()}Open in Simba{/snippet}
				</Tooltip>
			{/if}
		</div>
	{/if}
</footer>
