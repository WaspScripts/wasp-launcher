import { getLimits, getScriptContent } from "$lib/supabase"

export const load = async ({ params: { slug } }) => {
	return {
		details: Promise.all([getScriptContent(slug), getLimits(slug)])
	}
}
