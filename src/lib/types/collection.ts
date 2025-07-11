export interface Script {
	id: string
	url: string
	title: string
	description: string
	content: string
	protected: {
		username: string
		avatar: string
	}
}
