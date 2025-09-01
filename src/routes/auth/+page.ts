import { goto } from "$app/navigation"

export const load = async ({ parent }) => {
	const { session, profile } = await parent()
	console.log("🤖Loading auth page!")
	if (session && profile) {
		goto("/scripts")
	}

	return
}
