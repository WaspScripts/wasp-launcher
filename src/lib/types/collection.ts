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
	versions: {
		simba: string
		wasplib: string
	}
	stats_limits: {
		xp_min: number
		xp_max: number
		gp_min: number
		gp_max: number
	}
}
