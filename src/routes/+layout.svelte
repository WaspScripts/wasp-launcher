<script lang="ts">
	import { onMount } from "svelte"
	import "../app.css"
	import { invalidate } from "$app/navigation"

	let { data, children } = $props()
	const { supabase, session, dark, theme } = $derived(data)

	onMount(() => {
		document.documentElement.classList.toggle("dark", dark)
		document.body.setAttribute("data-theme", theme)

		const { data } = supabase.auth.onAuthStateChange((_, newSession) => {
			if (newSession?.expires_at !== session?.expires_at) {
				invalidate("supabase:auth")
			}
		})

		return () => data.subscription.unsubscribe()
	})
</script>

{@render children()}
