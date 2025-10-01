import type { Script } from "./types/collection"
import type { Database } from "./types/supabase"
import { createClient, type User } from "@supabase/supabase-js"

export const DATABASE_URL = "https://db.waspscripts.dev/"

export const supabase = createClient<Database>(
	DATABASE_URL,
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

export async function refreshSession() {
	console.log("Refreshing session")
	const { data, error } = await supabase.auth.refreshSession()
	if (error) {
		console.error(error)
		return
	}
	console.log("Session refreshed!", data)
}

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

export async function getSubscriptions(userID: string) {
	const { data, error: err } = await supabase
		.schema("profiles")
		.from("subscriptions")
		.select("product, price, date_start, date_end, cancel, disabled")
		.eq("user_id", userID)

	if (err) return []
	return data
}

export async function getFreeAccess(userID: string) {
	const { data, error: err } = await supabase
		.schema("profiles")
		.from("free_access")
		.select("product, date_start, date_end")
		.eq("id", userID)

	if (err) return []
	return data
}

export async function getBundles() {
	const { data, error: err } = await supabase
		.schema("scripts")
		.from("bundles")
		.select("id, author, name, scripts, username, avatar")
		.order("name")

	if (err) {
		console.error(err)
		return []
	}

	return data
}

export async function getProducts() {
	const { data, error: err } = await supabase
		.schema("stripe")
		.from("products")
		.select("id, user_id, bundle, script, name, username, avatar")
		.order("name")
		.eq("active", true)

	if (err) {
		console.error(err)
		return []
	}

	return data
}

export async function getScripts() {
	const { data, error: err } = await supabase
		.schema("scripts")
		.from("scripts")
		.select(
			`id, url, title, description, content,
			protected!left (username, avatar, revision, updated_at),
			metadata!left (status, type)`
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

export async function getData(profile: Exclude<Awaited<ReturnType<typeof getProfile>>, null>) {
	const promises = await Promise.all([
		getSubscriptions(profile.id),
		getFreeAccess(profile.id),
		getProducts(),
		getBundles(),
		getScripts()
	])

	const access = [...promises[0], ...promises[1]]
	const allproducts = promises[2]
	const bundles = promises[3]
	const scripts = promises[4]

	const bundleProducts = []
	const scriptProducts = []

	for (let i = 0; i < access.length; i++) {
		if (new Date(access[i].date_end).getTime() < Date.now()) continue
		for (let j = 0; j < allproducts.length; j++) {
			if (allproducts[j].id == access[i].product) {
				if (allproducts[j].bundle != null) bundleProducts.push(allproducts[j].bundle!)
				else scriptProducts.push(allproducts[j].script!)
				allproducts.splice(j, 1)
				break
			}
		}
	}

	for (let i = 0; i < bundleProducts.length; i++) {
		for (let j = 0; j < bundles.length; j++) {
			if (bundleProducts[i] != bundles[j].id) continue
			scriptProducts.push(...bundles[j].scripts)
			bundles.splice(j, 1)
			break
		}
	}

	const data = []
	if (
		profile.role == "tester" ||
		profile.role == "scripter" ||
		profile.role == "moderator" ||
		profile.role == "administrator"
	) {
		for (let i = 0; i < scripts.length; i++) data.push({ ...scripts[i], access: true })
	} else {
		for (let i = 0; i < scripts.length; i++) {
			if (scripts[i].metadata.type == "free") {
				data.push({ ...scripts[i], access: true })
				continue
			}

			data.push({ ...scripts[i], access: false })

			for (let j = 0; j < scriptProducts.length; j++) {
				if (scripts[i].id != scriptProducts[j]) continue
				data[i].access = true
				scriptProducts.splice(j, 1)
				break
			}
		}
	}

	return data
}
