<script lang="ts">
	import { invalidate } from "$app/navigation"
	import { ArrowBigLeft } from "@lucide/svelte"
	import { getVersion } from "@tauri-apps/api/app"
	import { invoke } from "@tauri-apps/api/core"
	import { open } from "@tauri-apps/plugin-dialog"

	let { data } = $props()

	async function getFile(exe: string, current: string) {
		let path = await open({
			title: "Pick a " + exe.toUpperCase() + " executable",
			defaultPath: current,
			multiple: false,
			directory: false,
			filters: [{ name: "Executables", extensions: ["exe"] }]
		})
		await invoke("set_executable_path", { exe, path })
		await invalidate("executable:paths")
	}
</script>

<main class="flex flex-col">
	<a
		href="/scripts"
		class="m-8 btn w-fit preset-outlined-surface-500 hover:text-primary-600-400"
		aria-label="Navigate to home page"
		data-sveltekit-preload-data="false"
	>
		<ArrowBigLeft />
	</a>

	<div class="m-12 space-y-8 rounded-md preset-outlined-surface-300-700 p-12">
		<header>
			<h1 class="text-xl font-bold">OSRS Clients:</h1>
		</header>

		<label class="label-text">
			RuneLite path:
			<input
				class="input w-96 preset-filled-surface-200-800"
				value={data.runelite}
				onclick={async () => await getFile("runelite", data.runelite!)}
			/>
		</label>

		<label class="label-text">
			OSClient path:
			<input
				class="input w-96 preset-filled-surface-200-800"
				value={data.osclient}
				onclick={async () => await getFile("osclient", data.osclient!)}
			/>
		</label>
	</div>

	<span class="w-full text-center font-bold text-surface-800-200">
		wasp-launcher v{#await getVersion()} Loading...{:then version}{version}{/await}</span
	>
</main>
