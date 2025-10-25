import { load as storeLoad } from "@tauri-apps/plugin-store"
import { getProfile, getSession, getUser, supabase } from "$lib/supabase"
import { error } from "@sveltejs/kit"
import { invoke } from "@tauri-apps/api/core"
export const prerender = true
export const ssr = false

export const load = async ({ depends, url: { searchParams } }) => {
	console.log("🚀Loading root layout!")
	const err = searchParams.get("error")
	if (err) error(403, "Login error: " + err)

	depends("supabase:auth")

	const promises = await Promise.all([
		getSession(),
		storeLoad("settings.json", { autoSave: true }),
		getProfile(getUser()),
		invoke("get_executable_path", { exe: "simba" }) as Promise<string>
	])

	const settings = promises[1]
	const themeSettings = await Promise.all([settings.get("dark"), settings.get("theme")])

	return {
		supabase,
		session: promises[0],
		profile: promises[2],
		simbaPath: promises[3],
		settings,
		dark: (themeSettings[0] as boolean) ?? true,
		theme: (themeSettings[1] as string) ?? "wasp"
	}
}
