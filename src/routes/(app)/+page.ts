import { goto } from "$app/navigation"

export const load = async ({ parent }) => {
	const { session, profile } = await parent()
	console.log("🔥Loading root page!")
	if (!session || !profile) goto("/auth")
	await goto("/scripts")
}
