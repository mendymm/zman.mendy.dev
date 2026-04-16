export interface City {
	id: string;
	n: string;
	cc: string;
	a1?: string | null;
	tz: string;
	lat: number;
	lon: number;
	elv: number;
	pop: number;
}

export interface NearestCityResult {
	city: City | null;
	distance: number;
}
