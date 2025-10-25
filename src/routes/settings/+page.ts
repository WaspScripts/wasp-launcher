import { goto } from "$app/navigation"
import { invoke } from "@tauri-apps/api/core"

export const load = async ({ parent, depends }) => {
	console.log("🔧Loading settings page!")
	depends("executable:paths")
	const promises = await Promise.all([
		parent(),
		invoke("get_executable_path", { exe: "devsimba" }) as Promise<string>,
		invoke("get_executable_path", { exe: "runelite" }) as Promise<string>,
		invoke("get_executable_path", { exe: "osclient" }) as Promise<string>
	])

	if (!promises[0].session) {
		goto("/auth")
		return
	}

	return {
		devsimba: promises[1],
		runelite: promises[2],
		osclient: promises[3],
		pluginVersions: invoke("get_plugin_version", {}) as Promise<string>
	}
}
