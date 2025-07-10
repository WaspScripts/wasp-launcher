import type { ProfileBase, ProfileRoles, Subscription, FreeAccess } from "$lib/types/collection"
import type { Database } from "$lib/types/supabase"
import type { Session, SupabaseClient, User } from "@supabase/supabase-js"

declare namespace App {
	// interface Locals {}
	interface PageData {
		supabase: SupabaseClient<Database>
		session: Session | null
		user: User | null
		profile: Promise<ProfileBase | null>
	}
	// interface Error {}
	// interface Platform {}
}
