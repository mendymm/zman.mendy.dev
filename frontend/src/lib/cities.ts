import type { City, NearestCityResult } from './types.js';

const ADMIN1_PATH = '/data/admin1.json';
const CITIES_PATH = '/data/cities.json';

export async function loadAdmin1(): Promise<Record<string, Record<string, string>>> {
	const res = await fetch(ADMIN1_PATH);
	if (!res.ok) throw new Error(`Failed to load admin1: ${res.status}`);
	return res.json() as Promise<Record<string, Record<string, string>>>;
}

export async function loadCities(): Promise<City[]> {
	const res = await fetch(CITIES_PATH);
	if (!res.ok) throw new Error(`Failed to load cities: ${res.status}`);
	const text = await res.text();
	return text
		.trim()
		.split('\n')
		.map((line) => JSON.parse(line) as City);
}

export function searchCities(cities: City[], query: string, limit = 20): City[] {
	if (!query) return [];
	const lowerQuery = query.toLowerCase();
	const matches: City[] = [];
	for (const city of cities) {
		if (city.n.toLowerCase().includes(lowerQuery)) {
			matches.push(city);
		}
	}
	matches.sort((a, b) => (b.pop || 0) - (a.pop || 0));
	return matches.slice(0, limit);
}

export function getAdmin1Name(
	admin1Names: Record<string, Record<string, string>>,
	countryCode: string,
	admin1Code?: string | null
): string | null {
	if (!admin1Code) return null;
	const countryMap = admin1Names[countryCode];
	if (!countryMap) return admin1Code;
	return countryMap[admin1Code] ?? admin1Code;
}

function calculateDistance(lat1: number, lon1: number, lat2: number, lon2: number): number {
	const R = 6371;
	const dLat = ((lat2 - lat1) * Math.PI) / 180;
	const dLon = ((lon2 - lon1) * Math.PI) / 180;
	const a =
		Math.sin(dLat / 2) * Math.sin(dLat / 2) +
		Math.cos((lat1 * Math.PI) / 180) *
			Math.cos((lat2 * Math.PI) / 180) *
			Math.sin(dLon / 2) *
			Math.sin(dLon / 2);
	return R * 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
}

export function findNearestCity(cities: City[], lat: number, lon: number): NearestCityResult {
	const RADIUS_KM = 30;
	const candidates: { city: City; distance: number }[] = [];

	for (const city of cities) {
		const distance = calculateDistance(lat, lon, city.lat, city.lon);
		if (distance <= RADIUS_KM) candidates.push({ city, distance });
	}

	if (candidates.length > 0) {
		candidates.sort((a, b) => b.city.pop - a.city.pop);
		return { city: candidates[0].city, distance: candidates[0].distance };
	}

	let nearest: City | null = null;
	let minDistance = Infinity;
	for (const city of cities) {
		const distance = calculateDistance(lat, lon, city.lat, city.lon);
		if (distance < minDistance) {
			minDistance = distance;
			nearest = city;
		}
	}
	return { city: nearest, distance: minDistance };
}
