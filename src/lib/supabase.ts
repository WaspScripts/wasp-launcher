import type { Database } from "./types/supabase"
import { createClient, type User } from "@supabase/supabase-js"

export const supabase = createClient<Database>(
	"https://db.waspscripts.dev",
	"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJzdXBhYmFzZSIsImlhdCI6MTc1MTA0MTIwMCwiZXhwIjo0OTA2NzE0ODAwLCJyb2xlIjoiYW5vbiJ9.C_KW5x45BpIyOQrnZc7CKYKjHe0yxB4l-fTSC4z_kYY",
	{
		auth: {
			persistSession: true,
			storageKey: "waspscripts-auth",
			storage: window.localStorage,
			detectSessionInUrl: true,
			flowType: "pkce"
		}
	}
)

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
