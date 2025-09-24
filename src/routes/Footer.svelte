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
	import { refreshSession } from "$lib/supabase"

	let data = $props()
	let script: ScriptEx = $derived(data.script)
	const supabase: SupabaseClient<Database> = $derived(page.data.supabase)
	const session: Session = $derived(page.data.session)

	function pad(n: number, size: number) {
		let s = n + ""
		while (s.length < size) s = "0" + s
		return s
	}

	async function saveBlobToFile(blob: Blob, path: string) {
		const arrayBuffer = await blob.arrayBuffer()
		const bytes = Array.from(new Uint8Array(arrayBuffer))

		await invoke("save_blob", {
			path,
			data: bytes
		})
	}

	async function getVersions(id: string) {
		const { data, error: err } = await supabase
			.schema("scripts")
			.from("versions")
			.select("revision, simba, wasplib")
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

	async function execute() {
		const versions = await versionsPromise
		const version = versions[selected]

		const { data, error: err } = await supabase.storage
			.from("scripts")
			.download(script.id + "/" + pad(version.revision, 9) + "/script.simba")
		if (err) {
			console.error(err)
			return
		}

		const file = script.url + "-rev-" + version.revision + ".simba"
		await saveBlobToFile(data, file)

		const exe = "simba"
		const args = [
			file,
			version.simba,
			version.wasplib,
			script.id,
			script.protected.revision.toString(),
			session.refresh_token
		]
		await invoke("run_executable", { exe, args })
		await refreshSession()
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
