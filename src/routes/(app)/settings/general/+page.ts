import { invoke } from "@tauri-apps/api/core"

export const load = async ({ depends }) => {
	console.log("🔧Loading settings page!")
	depends("executable:paths")
	const promises = await Promise.all([
		invoke("get_platform") as Promise<string>,
		invoke("get_simba_scale") as Promise<number>
	])

	return {
		platform: promises[0],
		simbaScale: promises[1]
	}
}
