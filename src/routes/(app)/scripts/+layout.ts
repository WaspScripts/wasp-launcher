import { redirect } from "@sveltejs/kit"

export const load = async ({ parent, params: { slug } }) => {
	const { scripts } = await parent()
	const script = scripts.find((script) => script.id === slug)
	if (!script) redirect(303, "/scripts/" + scripts[0].id)
	return { script }
}
