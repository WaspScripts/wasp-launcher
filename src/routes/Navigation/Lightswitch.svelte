<script lang="ts">
	import { page } from "$app/state"
	import { SunMoon } from "@lucide/svelte"

	const { settings, dark } = $derived(page.data)

	// svelte-ignore state_referenced_locally
	let current = $state(dark) as boolean

	async function toggleDarkMode() {
		current = !dark
		document.documentElement.classList.toggle("dark")
		await settings.set("dark", current)
	}
</script>

<button class="btn hover:preset-tonal my-auto" onclick={async () => await toggleDarkMode()}>
	<SunMoon />
</button>
