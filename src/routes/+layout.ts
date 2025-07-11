import { load as storeLoad } from "@tauri-apps/plugin-store"
import { getProfile, getSession, getUser, supabase } from "$lib/supabase"
import { error } from "@sveltejs/kit"
export const prerender = true
export const ssr = false

export const load = async ({ depends, url: { searchParams } }) => {
	const err = searchParams.get("error")
	if (err) error(403, "Login error: " + err)

	const promises = await Promise.all([getSession(), storeLoad("settings.json", { autoSave: true })])

	depends("supabase:auth")

	const user = getUser()
	const settings = promises[1]

	const themeSettings = await Promise.all([settings.get("dark"), settings.get("theme")])

	return {
		supabase,
		session: promises[0],
		user,
		profile: await getProfile(user),
		settings,
		dark: (themeSettings[0] as boolean) ?? true,
		theme: (themeSettings[1] as string) ?? "wasp"
	}
}
