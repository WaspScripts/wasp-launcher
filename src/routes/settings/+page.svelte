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
</script>

<main class="container mx-auto">
	<div class="my-12">
		<header>
			<h1>OSRS Clients:</h1>
		</header>

		<label class="my-4">
			RuneLite path:
			<input
				class="input"
				bind:value={data.runelite}
				onclick={async () => await getFile("runelite", data.runelite)}
			/>
		</label>

		<label class="my-4">
			OSClient path:
			<input
				class="input"
				bind:value={data.osclient}
				onclick={async () => await getFile("osclient", data.osclient)}
			/>
		</label>
	</div>
</main>
