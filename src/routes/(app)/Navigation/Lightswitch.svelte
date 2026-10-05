<script lang="ts">
	import { page } from "$app/state"
	import SunMoon from "@lucide/svelte/icons/sun-moon"

	const { settings, dark } = $derived(page.data)

	// svelte-ignore state_referenced_locally
	let current = $state(dark) as boolean

	async function toggleDarkMode() {
		current = !current
		document.documentElement.setAttribute("data-mode", current ? "dark" : "light")
		await settings.set("dark", current)
	}
</script>

<button class="my-auto btn hover:preset-tonal" onclick={async () => await toggleDarkMode()}>
	<SunMoon />
</button>
