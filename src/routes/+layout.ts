import { load as storeLoad } from "@tauri-apps/plugin-store"
import { getProfile, getSession, getUser, supabase } from "$lib/supabase"
import { error } from "@sveltejs/kit"
import { invoke } from "@tauri-apps/api/core"
import { assetsURLStore, devAssetsURLStore, devModeStore, devPathStore, devUpdatesStore } from "$lib/store"
export const prerender = true
export const ssr = false

export const load = async ({ depends, url: { searchParams } }) => {
	const err = searchParams.get("error")
	if (err) error(403, "Login error: " + err)

	depends("root:layout")

	const user = getUser()
	const promises = await Promise.all([
		getSession(user),
		storeLoad("settings.json", {
			autoSave: true,
			defaults: { dark: true, theme: "wasp", sidebar: true }
		}),
		getProfile(user),
		invoke("get_executable_path", { exe: "simba" }) as Promise<string>,
    invoke("get_executable_path", { exe: "devsimba" }) as Promise<string>,
		invoke("get_dev_mode") as Promise<boolean>,
    invoke("get_dev_updates") as Promise<boolean>,
    invoke("get_assets_url") as Promise<string>,
    invoke("get_dev_assets_url") as Promise<string>,
	])

	const settings = promises[1]
	const settingValues = await Promise.all([
		settings.get("dark"),
		settings.get("theme"),
		settings.get("sidebar")
	])

	devPathStore.set(promises[4])
	devModeStore.set(promises[5])
  devUpdatesStore.set(promises[6])
  assetsURLStore.set(promises[7])
  devAssetsURLStore.set(promises[8])

	return {
		supabase,
		session: promises[0],
		profile: promises[2],
		simbaPath: promises[3],
		settings,
		dark: (settingValues[0] as boolean) ?? true,
		theme: (settingValues[1] as string) ?? "wasp",
		sidebar: (settingValues[2] as boolean) ?? true
	}
}
