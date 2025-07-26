import { goto } from "$app/navigation"

export const prerender = "auto"

export const load = async ({ parent, params: { slug } }) => {
	const { session, scripts } = await parent()
	if (!session) {
		goto("/auth")
		return
	}

	if (!slug) {
		goto("/scripts/" + scripts[0].id)
	}

	const script = scripts.find((script) => script.id === slug)

	if (!script) {
		goto("/scripts/" + scripts[0].id)
	}

	return { scripts, script }
}
