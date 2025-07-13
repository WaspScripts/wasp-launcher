import { invoke } from "@tauri-apps/api/core"

export const load = async ({ depends }) => {
	depends("executable:paths")
	const promises = await Promise.all([
		invoke("get_executable_path", { exe: "simba" }) as Promise<string>,
		invoke("get_executable_path", { exe: "runelite" }) as Promise<string>,
		invoke("get_executable_path", { exe: "osclient" }) as Promise<string>
	])

	return {
		simba: promises[0],
		runelite: promises[1],
		osclient: promises[2]
	}
}
