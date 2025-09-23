import { goto } from "$app/navigation"
import { getData } from "$lib/supabase"

export const prerender = "auto"

export const load = async ({ parent, params: { slug } }) => {
	const { session, profile } = await parent()
	console.log("📜Loading scripts page!")
	if (!session || !profile) {
		goto("/auth")
		return
	}

	const scripts = await getData(profile)

	if (!slug) goto("/scripts/" + scripts[0].id)

	const script = scripts.find((script) => script.id === slug)

	if (!script) goto("/scripts/" + scripts[0].id)

	return { scripts, script }
}
