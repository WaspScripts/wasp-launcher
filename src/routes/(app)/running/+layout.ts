import { redirect } from "@sveltejs/kit"

export const load = async ({ parent }) => {
	const { scripts, script } = await parent()
	if (!script) redirect(303, "/running/" + scripts![0].id)
	return
}
