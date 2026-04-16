<script lang="ts">
	import { onMount } from "svelte";
	import type { City } from "$lib/types.js";
	import {
		loadCities,
		loadAdmin1,
		getAdmin1Name,
		findNearestCity,
	} from "$lib/cities.js";
	import { getCurrentPosition } from "$lib/geolocation.js";
	import { warmup, printZman } from "$lib";
	import CitySearch from "$lib/components/CitySearch.svelte";
	import ZmanimResult from "$lib/components/ZmanimResult.svelte";

	let cities = $state<City[]>([]);
	let admin1Names = $state<Record<string, Record<string, string>>>({});
	let dataLoaded = $state(false);
	let selectedCity = $state<City | null>(null);
	let selectedCityName = $state("");
	let date = $state(new Date().toISOString().split("T")[0]);
	let zmanimHtml = $state("");
	let locationStatus = $state("");

	onMount(async () => {
		void warmup();
		try {
			const [c, a] = await Promise.all([loadCities(), loadAdmin1()]);
			cities = c;
			admin1Names = a;
		} catch (e) {
			console.error("Failed to load data:", e);
		} finally {
			dataLoaded = true;
		}
	});

	function handleCitySelect(city: City) {
		selectedCity = city;
		selectedCityName = city.n;
		locationStatus = "";
		runCalculate(city);
	}

	async function handleGeolocation() {
		if (!dataLoaded) {
			locationStatus = "Please wait for city data to load...";
			return;
		}
		locationStatus = "Getting your location...";
		try {
			const { lat, lon } = await getCurrentPosition();
			const { city, distance } = findNearestCity(cities, lat, lon);
			if (city) {
				selectedCity = city;
				selectedCityName = city.n;
				const distText =
					distance < 1
						? `${Math.round(distance * 1000)} meters away`
						: `${distance.toFixed(1)} km away`;
				locationStatus = `Nearest city: ${city.n} (${distText})`;
				runCalculate(city);
			} else {
				locationStatus = "Could not find a nearby city";
			}
		} catch (e) {
			locationStatus = (e as Error).message;
		}
	}

	function runCalculate(city?: City | null) {
		const c = city ?? selectedCity;
		if (!c) {
			zmanimHtml = "<p>Please select a city first.</p>";
			return;
		}
		try {
			const admin1Display = getAdmin1Name(admin1Names, c.cc, c.a1);
			zmanimHtml = printZman(
				c.lat,
				c.lon,
				c.elv,
				c.tz,
				c.n,
				admin1Display,
				c.cc,
				date,
			);
		} catch (e) {
			zmanimHtml = `<div class="error">Error: ${(e as Error).message}</div>`;
		}
	}
</script>

<h1>Zmanim Calculator</h1>

<CitySearch
	{cities}
	{admin1Names}
	{dataLoaded}
	{selectedCityName}
	onselect={handleCitySelect}
/>

<div class="date-picker">
	<label for="date">Date:</label>
	<input type="date" id="date" bind:value={date} />
</div>

<div class="location-buttons">
	<button type="button" onclick={() => runCalculate()}
		>Calculate Zmanim</button
	>
	<button type="button" onclick={() => void handleGeolocation()}
		>Use My Location</button
	>
</div>
<div id="locationStatus">{locationStatus}</div>

<h2>Results</h2>
<ZmanimResult html={zmanimHtml} {dataLoaded} />

<footer class="footer">
	Copyright &copy; 2026 Mendy Man. Licensed under
	<a href="/LICENSE.txt" target="_blank">LGPL-3.0-or-later</a>.<br />
	Zmanim calculations powered by
	<a href="https://github.com/YSCohen/rust-zmanim" target="_blank"
		>rust-zmanim</a
	>.<br />
	For corrections/comments/notes please email me at
	<a href="mailto:zman@mendy.dev">zman@mendy.dev</a>
</footer>
