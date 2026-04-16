<script lang="ts">
	import type { City } from '$lib/types.js';
	import { searchCities, getAdmin1Name } from '$lib/cities.js';
	import { getCurrentTime } from '$lib';

	interface Props {
		cities: City[];
		admin1Names: Record<string, Record<string, string>>;
		dataLoaded: boolean;
		selectedCityName: string;
		onselect: (city: City) => void;
	}

	let { cities, admin1Names, dataLoaded, selectedCityName, onselect }: Props = $props();

	let query = $state('');
	let activeIndex = $state(-1);
	let showDropdown = $state(false);

	// When the parent pushes a city name (e.g. via geolocation), sync the input.
	$effect(() => {
		query = selectedCityName;
		showDropdown = false;
		activeIndex = -1;
	});

	const results = $derived(dataLoaded && query.trim() ? searchCities(cities, query) : []);

	// city.id → local time string, populated lazily via WASM
	let timeCache = $state(new Map<string, string>());

	$effect(() => {
		const r = results;
		if (r.length === 0) return;
		const now = BigInt(Date.now());
		for (const city of r) {
			if (!timeCache.has(city.id)) {
				try {
					timeCache.set(city.id, getCurrentTime(city.tz, now));
				} catch {
					timeCache.set(city.id, '');
				}
			}
		}
	});

	function handleInput(e: Event) {
		query = (e.target as HTMLInputElement).value;
		activeIndex = -1;
		showDropdown = Boolean(query.trim());
	}

	function handleKeydown(e: KeyboardEvent) {
		if (!showDropdown || results.length === 0) return;
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			activeIndex = activeIndex >= results.length - 1 ? 0 : activeIndex + 1;
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			activeIndex = activeIndex <= 0 ? results.length - 1 : activeIndex - 1;
		} else if (e.key === 'Enter') {
			e.preventDefault();
			if (activeIndex >= 0 && activeIndex < results.length) {
				selectCity(results[activeIndex]);
			}
		} else if (e.key === 'Escape') {
			showDropdown = false;
			activeIndex = -1;
		}
	}

	function selectCity(city: City) {
		query = city.n;
		showDropdown = false;
		activeIndex = -1;
		onselect(city);
	}

	function formatLocation(city: City): string {
		const admin1Name = getAdmin1Name(admin1Names, city.cc, city.a1);
		return admin1Name ? `${city.n}, ${admin1Name}, ${city.cc}` : `${city.n}, ${city.cc}`;
	}
</script>

<div class="search-container">
	<input
		type="text"
		id="search"
		placeholder={dataLoaded ? 'Search for a city...' : 'Loading city data...'}
		disabled={!dataLoaded}
		autocomplete="off"
		value={query}
		oninput={handleInput}
		onkeydown={handleKeydown}
		onblur={() =>
			setTimeout(() => {
				showDropdown = false;
				activeIndex = -1;
			}, 150)}
	/>
	{#if showDropdown && results.length > 0}
		<div class="autocomplete-items">
			{#each results as city, i (city.id)}
				<div
					class:autocomplete-active={i === activeIndex}
					role="option"
					aria-selected={i === activeIndex}
					tabindex="-1"
					onmousedown={(e) => {
						e.preventDefault();
						selectCity(city);
					}}
				>
					<span class="city-name">{formatLocation(city)}</span>
					<span class="city-time">{timeCache.get(city.id) ?? '...'}</span>
				</div>
			{/each}
		</div>
	{/if}
</div>
