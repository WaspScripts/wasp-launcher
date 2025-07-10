<script lang="ts">
	import { onMount } from "svelte"
	import "../app.css"
	import { invalidate } from "$app/navigation"
	import Login from "./Login.svelte"
	import Navigation from "./Navigation/Navigation.svelte"

	let { data, children } = $props()
	const { supabase, session } = $derived(data)

	onMount(() => {
		const { data } = supabase.auth.onAuthStateChange((_, newSession) => {
			if (newSession?.expires_at !== session?.expires_at) {
				invalidate("supabase:auth")
			}
		})

		return () => data.subscription.unsubscribe()
	})
</script>

{#if session}
	<Navigation />
	{@render children()}
{:else}
	<main class="container h-screen mx-auto flex justify-center items-center">
		<Login />
	</main>
{/if}
