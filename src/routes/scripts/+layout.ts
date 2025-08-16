import { goto } from "$app/navigation"
import type { Script } from "$lib/types/collection"

export const prerender = "auto"

export const load = async ({ parent, params: { slug } }) => {
	const { supabase, session } = await parent()
	if (!session) goto("/auth")

	async function getScripts() {
		const { data, error: err } = await supabase
			.schema("scripts")
			.from("scripts")
			.select(
				"id, url, title, description, content, protected!left (username, avatar, revision, updated_at), stats_limits!left (xp_min, xp_max, gp_min, gp_max)"
			)
			.eq("published", true)
			.order("title")
			.overrideTypes<Script[]>()

		if (err) {
			console.error(err)
			return []
		}

		return data
	}

	const scripts = await getScripts()

	if (!slug) goto("/scripts/" + scripts[0].id)

	const script = scripts.find((script) => script.id === slug)

	if (!script) goto("/scripts/" + scripts[0].id)

	return { scripts, script }
}
