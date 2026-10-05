<script lang="ts">
	import { onMount } from "svelte"
	import "../app.css"
	import { invalidate } from "$app/navigation"
	import { listen } from "@tauri-apps/api/event"
	import { channelManager } from "$lib/communication.svelte"

	let { data, children } = $props()
	const { supabase, session } = $derived(data)

	let callTimestamps: number[] = []
	onMount(() => {
		const unlisten = listen<string>("process-finished", async (event) => {
			const channel = Number(event.payload)
			console.log(`Process finished: ${channel}`)
			await Promise.all([channelManager.stopChannel(channel), invalidate("layout:running")])
		})

		const { data } = supabase.auth.onAuthStateChange((_, newSession) => {
			if (newSession?.expires_at !== session?.expires_at) {
				const now = Date.now()
				callTimestamps = callTimestamps.filter((ts) => now - ts < 10000)
				if (callTimestamps.length >= 10) {
					console.error(
						"Rate limit exceeded: invalidate('root:layout') blocked to prevent loop infinite loop."
					)
					return
				}
				callTimestamps.push(now)
				invalidate("root:layout")
			}
		})

		return () => {
			data.subscription.unsubscribe()
			unlisten.then((fn) => fn())
		}
	})
</script>

{@render children()}
