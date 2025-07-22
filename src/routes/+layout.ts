import { load as storeLoad } from "@tauri-apps/plugin-store"
import { getProfile, getSession, getUser, supabase } from "$lib/supabase"
import { error } from "@sveltejs/kit"
import type { Script } from "$lib/types/collection"
export const prerender = true
export const ssr = false

export const load = async ({ depends, url: { searchParams } }) => {
	const err = searchParams.get("error")
	if (err) error(403, "Login error: " + err)

	depends("supabase:auth")
	console.log("Reloading root layout!")

	async function getScripts() {
		const { data, error: err } = await supabase
			.schema("scripts")
			.from("scripts")
			.select(
				"id, url, title, description, content, protected!left (username, avatar, revision), stats_limits!left (xp_min, xp_max, gp_min, gp_max)"
			)
			.eq("published", true)
			.order("title")
			.overrideTypes<Script[]>()

		if (err) {
			console.error(err)
			return []
		}

		return data
	}

	const promises = await Promise.all([
		getSession(),
		storeLoad("settings.json", { autoSave: true }),
		getScripts()
	])

	const user = getUser()
	const settings = promises[1]

	const themeSettings = await Promise.all([settings.get("dark"), settings.get("theme")])

	return {
		supabase,
		session: promises[0],
		user,
		profile: await getProfile(user),
		scripts: promises[2],
		settings,
		dark: (themeSettings[0] as boolean) ?? true,
		theme: (themeSettings[1] as string) ?? "wasp"
	}
}
