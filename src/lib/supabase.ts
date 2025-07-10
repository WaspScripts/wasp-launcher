import { PUBLIC_SUPABASE_ANON_KEY, PUBLIC_SUPABASE_URL } from "$env/static/public"
import type { Database } from "./types/supabase"
import { createClient, type User } from "@supabase/supabase-js"

export const supabase = createClient<Database>(PUBLIC_SUPABASE_URL, PUBLIC_SUPABASE_ANON_KEY, {
	auth: {
		persistSession: true,
		storageKey: "waspscripts-auth",
		storage: window.localStorage,
		detectSessionInUrl: true,
		flowType: "pkce"
	}
})

export async function getSession() {
	const {
		data: { session }
	} = await supabase.auth.getSession()

	return session
}

export async function getUser() {
	const {
		data: { user }
	} = await supabase.auth.getUser()

	return user
}

export async function getProfile(userPromise: Promise<User | null>) {
	const user = await userPromise
	if (!user) return null
	const { data, error: err } = await supabase
		.schema("profiles")
		.from("profiles")
		.select(`id, discord, stripe, username, avatar, role`)
		.eq("id", user.id)
		.single()

	if (err) return null

	return data
}
