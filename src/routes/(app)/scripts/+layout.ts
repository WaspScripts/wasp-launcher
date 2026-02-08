import { goto } from "$app/navigation"

export const load = async ({ parent }) => {
	const { scripts, script } = await parent()
	if (!script) await goto("/scripts/" + scripts![0].id)
	return
}
