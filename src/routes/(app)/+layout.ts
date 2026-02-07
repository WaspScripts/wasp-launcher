import { goto } from "$app/navigation"
import { getData } from "$lib/supabase"

export const load = async ({ parent, params: { slug } }) => {
	const { session, profile } = await parent()
	console.log("📜Loading scripts page!")
	if (!session || !profile) {
		await goto("/auth")
		return
	}

	const scripts = await getData(profile)
	const script = scripts.find((script) => script.id === slug)

	return { scripts, script }
}
