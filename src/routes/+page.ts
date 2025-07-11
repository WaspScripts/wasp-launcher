import { getScripts } from "$lib/supabase"

export const load = async () => {
	return { scripts: await getScripts() }
}
