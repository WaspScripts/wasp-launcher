import { getData } from "$lib/supabase"
import { redirect } from "@sveltejs/kit"

export const load = async ({ parent }) => {
	const { session, profile } = await parent()
	console.log("📜Loading scripts page!")
	if (!session || !profile) {
		redirect(303, "/auth")
	}

	const scripts = await getData(profile)
	if (scripts.length === 0) redirect(303, "/auth")

	return { scripts }
}
