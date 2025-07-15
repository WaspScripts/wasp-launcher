import { goto } from "$app/navigation"

export const load = async ({ parent, params: { slug } }) => {
	const { session, scripts } = await parent()
	if (!session) {
		goto("/auth")
		return
	}

	if (!slug) {
		goto("/scripts/" + scripts[0].url)
	}

	return { scripts }
}
