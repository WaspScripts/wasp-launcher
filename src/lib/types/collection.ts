export interface Script {
	id: string
	url: string
	title: string
	description: string
	content: string
	protected: {
		username: string
		avatar: string
		revision: number
		updated_at: number
	}
	metadata: {
		status: "official" | "community"
		type: "premium" | "free"
	}
}

export interface StatsLimits {
	xp_min: number
	xp_max: number
	gp_min: number
	gp_max: number
}

export type ScriptEx = Script & {
	access: boolean
}
