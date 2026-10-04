import { invoke } from "@tauri-apps/api/core"

export const load = async ({ depends }) => {
	console.log("🔧Loading settings page!")
	depends("executable:paths")
	const promises = await Promise.all([
		invoke("get_executable_path", { exe: "runelite" }) as Promise<string>,
		invoke("get_executable_path", { exe: "osclient" }) as Promise<string>,
		invoke("get_platform") as Promise<string>,
		invoke("get_simba_scale") as Promise<number>
	])

	return {
		runelite: promises[0],
		osclient: promises[1],
		platform: promises[2],
		simbaScale: promises[3]
	}
}
