import { getProfile, getSession, getUser, supabase } from "$lib/supabase"
import { error } from "@sveltejs/kit"
export const prerender = true
export const ssr = false

export const load = async ({ depends, url: { searchParams } }) => {
	const err = searchParams.get("error")
	if (err) error(403, "Login error: " + err)

	depends("supabase:auth")
	const user = getUser()
	return { supabase, session: await getSession(), user, profile: getProfile(user) }
}
