import { goto } from "$app/navigation"

export const load = async ({ parent }) => {
	const { session } = await parent()
	if (!session) goto("/auth")
	goto("/scripts")
}
