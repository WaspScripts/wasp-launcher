<script lang="ts">
	import { supabase } from "../lib/supabase"
	import Discord from "../lib/components/Discord.svelte"
	import Logo from "../lib/components/Logo.svelte"
	import { invoke } from "@tauri-apps/api/core"
	import { openUrl } from "@tauri-apps/plugin-opener"
	import { listen } from "@tauri-apps/api/event"
	import { goto } from "$app/navigation"

	async function login() {
		console.log("Logging in with discord...")
		const { data, error } = await supabase.auth.signInWithOAuth({
			provider: "discord",
			options: { skipBrowserRedirect: true, redirectTo: `http://localhost:5217/` }
		})

		if (error) {
			console.error(error)
			return
		}

		if (data.url) {
			await invoke("start_server")
			await openUrl(data.url)

			listen("oauth-callback", async (event) => {
				const { code, error } = event.payload as { code?: string; error?: string }
				if (code) {
					console.log("OAuth code:", code)
					const {
						data: { user },
						error: err
					} = await supabase.auth.exchangeCodeForSession(code)

					if (err) {
						console.error(err)
						return
					}

					if (!user) {
						console.error("Failed to get user.")
						return
					}

					const { count } = await supabase
						.schema("profiles")
						.from("profiles")
						.select("*", { count: "exact", head: true })
						.eq("id", user.id)
						.single()

					if (count) {
						goto("/")
						return
					}

					console.error("No profile! Please login on WaspScripts website first.")
				} else {
					console.error("OAuth error:", error)
				}
			})
		}
	}
</script>

<div class="mx-auto my-24 flex flex-col">
	<Logo />
	<form class="flex my-20 items-center" on:submit={async () => await login()}>
		<button type="submit" class="btn preset-filled-surface-300-700 py-2 hover:preset-tonal">
			<Discord />
			<span> Login with Discord </span>
		</button>
	</form>
</div>
