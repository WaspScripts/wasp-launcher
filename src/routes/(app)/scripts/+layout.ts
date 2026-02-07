import { goto } from "$app/navigation"
export const prerender = true
export const ssr = false

export const load = async ({ parent }) => {
	const { scripts, script } = await parent()
	if (!scripts || scripts.length == 0) await goto("/error")
	if (!script) await goto("/scripts/" + scripts![0].id)
	return
}
