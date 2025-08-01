import { goto } from "$app/navigation"
import { invoke } from "@tauri-apps/api/core"

export const load = async ({ parent, depends }) => {
	depends("executable:paths")
	const promises = await Promise.all([
		parent(),
		invoke("get_executable_path", { exe: "simba" }) as Promise<string>,
		invoke("get_executable_path", { exe: "runelite" }) as Promise<string>,
		invoke("get_executable_path", { exe: "osclient" }) as Promise<string>
	])

	if (!promises[0].session) {
		goto("/auth")
		return
	}

	return {
		simba: promises[1],
		runelite: promises[2],
		osclient: promises[3]
	}
}
