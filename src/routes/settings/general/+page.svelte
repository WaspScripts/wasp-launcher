<script lang="ts">
	import { invalidate } from "$app/navigation"
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

	let deletingCache = $state(false)
	async function deleteCache() {
		deletingCache = true
		await invoke("delete_cache", { exe: "simba" })
		deletingCache = false
	}

	let deletingAssets = $state(false)
	async function deleteAssets() {
		deletingAssets = true
		await invoke("delete_assets", { exe: "simba" })
		deletingAssets = false
	}

	let deletingConfigs = $state(false)
	async function deleteConfigs() {
		deletingConfigs = true
		await invoke("delete_configs", { exe: "simba" })
		deletingConfigs = false
	}

	let dialog: HTMLDialogElement
	let reinstallingPlugins = $state(false)
	async function reinstallPlugins() {
		dialog.close()
		reinstallingPlugins = true
		await invoke("reinstall_plugins", { exe: "simba" })
		reinstallingPlugins = false
	}
</script>

<main class="flex flex-col">
	<div class="mx-auto my-4 space-y-8 rounded-md preset-outlined-surface-300-700 p-12">
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

	<div class="mx-auto my-4 flex gap-2">
		<button
			class="btn preset-filled-primary-500 font-bold"
			class:disabled={deletingCache}
			disabled={deletingCache}
			onclick={async () => await deleteCache()}
		>
			Clear Cache
		</button>
		<button
			class="btn preset-filled-primary-500 font-bold"
			class:disabled={deletingAssets}
			disabled={deletingAssets}
			onclick={async () => await deleteAssets()}
		>
			Clear Assets
		</button>
		<button
			class="btn preset-filled-primary-500 font-bold"
			class:disabled={deletingConfigs}
			disabled={deletingConfigs}
			onclick={async () => await deleteConfigs()}
		>
			Clear Configs
		</button>

		<dialog
			bind:this={dialog}
			data-dialog
			class="top-1/2 left-1/2 z-10 max-w-[640px] -translate-1/2 space-y-4 rounded-container bg-surface-100-900 p-4 text-inherit backdrop:bg-surface-50/75 dark:backdrop:bg-surface-950/75"
		>
			<h2 class="h3">Reinstall Plugins</h2>
			<p>Please make sure you are not running any client you've used waspscripts on.</p>
			<p>
				If you are not sure close all clients you have open, check the task manager to be sure none
				is running in the background.
			</p>
			<footer class="flex justify-end gap-4">
				<button type="button" class="btn preset-tonal" onclick={() => dialog.close()}>
					Cancel
				</button>
				<button class="btn preset-filled" onclick={reinstallPlugins}> Confirm </button>
			</footer>
		</dialog>

		<button
			class="btn preset-filled-primary-500 font-bold"
			disabled={reinstallingPlugins}
			onclick={() => dialog.show()}
		>
			Reinstall plugins
		</button>
	</div>
</main>
