<script lang="ts">
	import { invalidate } from "$app/navigation"
	import { ArrowBigLeft } from "@lucide/svelte"
	import { getVersion } from "@tauri-apps/api/app"
	import { invoke } from "@tauri-apps/api/core"
	import { open } from "@tauri-apps/plugin-dialog"
	import { Modal } from "@skeletonlabs/skeleton-svelte"

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

	async function getPath(exe: string, current: string) {
		let path = await open({
			title: "Pick a " + exe.toUpperCase() + " directory",
			defaultPath: current,
			multiple: false,
			directory: true,
			filters: [{ name: "Directories", extensions: [] }]
		})
		await invoke("set_executable_path", { exe, path })
		await invalidate("executable:paths")
	}

	let deletingAssets = $state(false)
	async function clearAssets() {
		deletingAssets = true
		await invoke("delete_assets")
		deletingAssets = false
	}

	let deletingCache = $state(false)
	async function clearCache() {
		deletingCache = true
		await invoke("delete_cache")
		deletingCache = false
	}

	let deletingConfigs = $state(false)
	async function clearConfigs() {
		deletingConfigs = true
		await invoke("delete_configs")
		deletingConfigs = false
	}

	let reinstallingPlugins = $state(false)

	let modalOpenState = $state(false)

	function modalClose() {
		modalOpenState = false
	}

	async function reinstallPlugins() {
		modalOpenState = false
		reinstallingPlugins = true
		await invoke("reinstall_plugins")
		reinstallingPlugins = false
	}
</script>

<main class="flex flex-col">
	<a
		href="/scripts"
		class="mx-8 my-4 btn w-fit preset-outlined-surface-500 hover:text-primary-600-400"
		aria-label="Navigate to home page"
		data-sveltekit-preload-data="false"
	>
		<ArrowBigLeft />
	</a>

	<div class="flex gap-2">
		<div class="m-4 space-y-8 rounded-md preset-outlined-surface-300-700 p-12">
			<header>
				<h1 class="text-xl font-bold">OSRS Clients:</h1>
			</header>

			<label class="label-text">
				RuneLite path:
				<input
					class="input w-96 preset-filled-surface-200-800 hover:outline-1 hover:outline-primary-500"
					value={data.runelite}
					onclick={async () => await getFile("runelite", data.runelite!)}
				/>
			</label>

			<label class="label-text">
				OSClient path:
				<input
					class="input w-96 preset-filled-surface-200-800 hover:outline-1 hover:outline-primary-500"
					value={data.osclient}
					onclick={async () => await getFile("osclient", data.osclient!)}
				/>
			</label>
		</div>

		<div class="m-4 space-y-8 rounded-md preset-outlined-surface-300-700 p-12">
			<header>
				<h1 class="text-xl font-bold">Development Environment:</h1>
			</header>

			<p class="text-sm">
				This allows you to setup a dev environment on the directory you pick below.
			</p>
			<p class="text-sm">
				You can fully update it and open Simba in that directory by clicking the lion icon on the
				right side panel.
			</p>
			<label class="label-text">
				Dev environment:
				<input
					class="input w-96 preset-filled-surface-200-800 hover:outline-1 hover:outline-primary-500"
					value={data.devsimba}
					onclick={async () => await getPath("devsimba", data.devsimba!)}
				/>
			</label>
		</div>
	</div>

	<div class="mx-auto my-4 flex gap-2">
		<button
			class="btn preset-filled-primary-500 font-bold"
			class:disabled={deletingCache}
			disabled={deletingCache}
			onclick={async () => await clearCache()}
		>
			Clear Cache
		</button>
		<button
			class="btn preset-filled-primary-500 font-bold"
			class:disabled={deletingAssets}
			disabled={deletingAssets}
			onclick={async () => await clearAssets()}
		>
			Clear Assets
		</button>
		<button
			class="btn preset-filled-primary-500 font-bold"
			class:disabled={deletingConfigs}
			disabled={deletingConfigs}
			onclick={async () => await clearConfigs()}
		>
			Clear Configs
		</button>

		<Modal
			open={modalOpenState}
			onOpenChange={(e) => (modalOpenState = e.open)}
			triggerBase=""
			contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
			backdropClasses="backdrop-blur-sm"
		>
			{#snippet trigger()}
				<button
					class="btn preset-filled-primary-500 font-bold"
					class:disabled={reinstallingPlugins}
					disabled={reinstallingPlugins}
				>
					Reinstall plugins
				</button>
			{/snippet}
			{#snippet content()}
				<header class="flex justify-between">
					<h2 class="h2">Reinstall Plugins</h2>
				</header>
				<article class="my-16 space-y-8">
					<p>Please make sure you are not running any client you've used waspscripts on.</p>
					<p>
						If you are not sure close all clients you have open, check the task manager to be sure
						none is running in the background.
					</p>
				</article>
				<footer class="flex justify-end gap-4">
					<button type="button" class="btn preset-tonal" onclick={modalClose}>Cancel</button>
					<button type="button" class="btn preset-filled" onclick={reinstallPlugins}>Confirm</button
					>
				</footer>
			{/snippet}
		</Modal>
	</div>

	<div class="my-4 w-full space-x-8 text-center font-bold text-surface-800-200">
		<span>
			wasp-launcher v{#await getVersion()} Loading...{:then version}{version}{/await}
		</span>
		<span>
			wasp-plugins v{#await data.pluginVersions} Loading...{:then version}{version}{/await}
		</span>
	</div>
</main>
