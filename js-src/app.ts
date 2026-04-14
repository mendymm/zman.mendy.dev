import type { City, NearestCityResult } from "./types";
import { initSync, dbg_zemanim, get_current_time } from "../wasm-funcs/pkg/wasm_funcs";

const ADMIN1_PATH = "/data/admin1.json.br";
const CITIES_PATH = "/data/cities.jsonl.br";
const WASM_PATH = "/dist/wasm_funcs_bg.wasm.br";

let admin1Names: Record<string, Record<string, string>> = {};
let cities: City[] = [];
let wasmReady = false;
let selectedCity: City | null = null;
let isLoading = true;

async function fetchText(path: string): Promise<string> {
  const response = await fetch(path);
  if (!response.ok) {
    throw new Error(`Failed to fetch ${path}: ${response.status}`);
  }
  return response.text();
}

async function fetchJson<T>(path: string): Promise<T> {
  const response = await fetch(path);
  if (!response.ok) {
    throw new Error(`Failed to fetch ${path}: ${response.status}`);
  }
  return response.json();
}

async function fetchArrayBuffer(path: string): Promise<ArrayBuffer> {
  const response = await fetch(path);
  if (!response.ok) {
    throw new Error(`Failed to fetch ${path}: ${response.status}`);
  }
  return response.arrayBuffer();
}

function getAdmin1Name(countryCode: string, admin1Code?: string | null): string | null {
  if (!admin1Code) return null;
  const countryMap = admin1Names[countryCode];
  if (!countryMap) return admin1Code;
  return countryMap[admin1Code] || admin1Code;
}

async function loadAdmin1(): Promise<void> {
  try {
    admin1Names = await fetchJson(ADMIN1_PATH);
    console.log("Loaded admin1 names");
  } catch (e) {
    console.log("Failed to load admin1 names:", (e as Error).message);
  }
}

async function loadCities(): Promise<void> {
  const searchInput = document.getElementById("search") as HTMLInputElement;
  const resultDiv = document.getElementById("zmanim_result") as HTMLDivElement;

  searchInput.disabled = true;
  searchInput.placeholder = "Loading city data...";
  resultDiv.textContent = "Loading city data, please wait...";

  try {
    const text = await fetchText(CITIES_PATH);
    cities = text
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line) as City);
    resultDiv.textContent = `Loaded ${cities.length} cities. Search city above.`;
    resultDiv.classList.remove("loading");
  } catch (e) {
    resultDiv.textContent = "Error loading city data: " + (e as Error).message;
  } finally {
    isLoading = false;
    searchInput.disabled = false;
    searchInput.placeholder = "Search for a city...";
  }
}

function searchCities(query: string, limit = 20): City[] {
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

function selectCity(city: City): void {
  selectedCity = city;
  (document.getElementById("search") as HTMLInputElement).value = city.n;
  (document.getElementById("lat") as HTMLInputElement).value = String(city.lat);
  (document.getElementById("lon") as HTMLInputElement).value = String(city.lon);
  (document.getElementById("elevation") as HTMLInputElement).value = String(city.elv);
  (document.getElementById("tz") as HTMLInputElement).value = city.tz;
  document.getElementById("autocomplete-list")!.innerHTML = "";
  document.getElementById("locationStatus")!.textContent = "";
  calculateZmanim();
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
  const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
  return R * c;
}

function findNearestCity(lat: number, lon: number): NearestCityResult {
  const RADIUS_KM = 30;
  const candidates: { city: City; distance: number }[] = [];

  for (const city of cities) {
    const distance = calculateDistance(lat, lon, city.lat, city.lon);
    if (distance <= RADIUS_KM) {
      candidates.push({ city, distance });
    }
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

function handleGeolocation(): void {
  const statusDiv = document.getElementById("locationStatus") as HTMLDivElement;

  if (!navigator.geolocation) {
    statusDiv.textContent = "Geolocation is not supported by your browser";
    return;
  }

  if (isLoading || cities.length === 0) {
    statusDiv.textContent = "Please wait for city data to load...";
    return;
  }

  statusDiv.textContent = "Getting your location...";

  navigator.geolocation.getCurrentPosition(
    (position) => {
      console.log(position);
      const lat = position.coords.latitude;
      const lon = position.coords.longitude;

      const { city, distance } = findNearestCity(lat, lon);

      if (city) {
        selectedCity = city;
        (document.getElementById("search") as HTMLInputElement).value = city.n;
        (document.getElementById("lat") as HTMLInputElement).value = String(city.lat);
        (document.getElementById("lon") as HTMLInputElement).value = String(city.lon);
        (document.getElementById("elevation") as HTMLInputElement).value = String(city.elv);
        (document.getElementById("tz") as HTMLInputElement).value = city.tz;

        const distanceText =
          distance < 1
            ? `${Math.round(distance * 1000)} meters away`
            : `${distance.toFixed(1)} km away`;

        statusDiv.textContent = `Nearest city: ${city.n} (${distanceText})`;
        calculateZmanim();
      } else {
        statusDiv.textContent = "Could not find a nearby city";
      }
    },
    (error) => {
      let message = "Unable to get your location";
      switch (error.code) {
        case error.PERMISSION_DENIED:
          message = "Location permission denied. Please enable location access.";
          break;
        case error.POSITION_UNAVAILABLE:
          message = "Location information unavailable";
          break;
        case error.TIMEOUT:
          message = "Location request timed out";
          break;
      }
      statusDiv.textContent = message;
    },
    {
      enableHighAccuracy: true,
      timeout: 10000,
      maximumAge: 0,
    },
  );
}

function calculateZmanim(): void {
  if (!wasmReady) {
    document.getElementById("zmanim_result")!.textContent = "WASM not ready yet...";
    return;
  }

  const latStr = (document.getElementById("lat") as HTMLInputElement).value;
  const lonStr = (document.getElementById("lon") as HTMLInputElement).value;

  if (!latStr || !lonStr) {
    document.getElementById("zmanim_result")!.textContent =
      "Please select a city or use your location first.";
    return;
  }

  const lat = parseFloat(latStr);
  const lon = parseFloat(lonStr);

  if (isNaN(lat) || isNaN(lon)) {
    document.getElementById("zmanim_result")!.textContent =
      "Invalid location data. Please select a city.";
    return;
  }
  const elevation = parseFloat((document.getElementById("elevation") as HTMLInputElement).value);
  const tz = (document.getElementById("tz") as HTMLInputElement).value;
  const locationName = selectedCity?.n || "Custom Location";
  const admin1Code = selectedCity ? getAdmin1Name(selectedCity.cc, selectedCity.a1) : null;
  const countryCode = selectedCity?.cc || "";
  const dateStr = (document.getElementById("date") as HTMLInputElement).value || null;

  try {
    const result = dbg_zemanim(
      lat,
      lon,
      elevation,
      tz,
      locationName,
      admin1Code,
      countryCode,
      dateStr,
    );
    document.getElementById("zmanim_result")!.innerHTML = result;
  } catch (e) {
    document.getElementById("zmanim_result")!.innerHTML =
      `<div class="error">Error: ${(e as Error).message}</div>`;
  }
}

function setupAutocomplete(): void {
  const input = document.getElementById("search") as HTMLInputElement;
  const list = document.getElementById("autocomplete-list") as HTMLDivElement;
  let currentFocus = -1;

  input.addEventListener("input", function () {
    if (isLoading) return;

    const val = this.value;
    list.innerHTML = "";
    currentFocus = -1;

    if (!val) return;

    const matches = searchCities(val);
    matches.forEach((city) => {
      const div = document.createElement("div");
      const currentTime = wasmReady ? get_current_time(city.tz, BigInt(Date.now())) : "...";
      const admin1Name = getAdmin1Name(city.cc, city.a1);
      const location = admin1Name
        ? `${city.n}, ${admin1Name}, ${city.cc}`
        : `${city.n}, ${city.cc}`;
      div.innerHTML = `<span class="city-name">${location}</span> <span class="city-time">${currentTime}</span>`;
      div.addEventListener("click", () => selectCity(city));
      list.appendChild(div);
    });
  });

  input.addEventListener("keydown", function (e) {
    const items = list.getElementsByTagName("div");
    if (e.keyCode === 40) {
      currentFocus++;
      addActive(items);
      e.preventDefault();
    } else if (e.keyCode === 38) {
      currentFocus--;
      addActive(items);
      e.preventDefault();
    } else if (e.keyCode === 13) {
      e.preventDefault();
      if (currentFocus > -1 && items[currentFocus]) {
        items[currentFocus].click();
      }
    }
  });

  function addActive(items: HTMLCollectionOf<HTMLDivElement>): void {
    if (!items) return;
    removeActive(items);
    if (currentFocus >= items.length) currentFocus = 0;
    if (currentFocus < 0) currentFocus = items.length - 1;
    items[currentFocus].classList.add("autocomplete-active");
  }

  function removeActive(items: HTMLCollectionOf<HTMLDivElement>): void {
    for (const item of items) {
      item.classList.remove("autocomplete-active");
    }
  }

  document.addEventListener("click", function (e) {
    if (e.target !== input) {
      list.innerHTML = "";
    }
  });
}

async function loadWasm(): Promise<void> {
  try {
    const wasmBuffer = await fetchArrayBuffer(WASM_PATH);
    initSync(new Uint8Array(wasmBuffer));
    console.log("WASM loaded");
  } catch (e) {
    console.log("WASM load failed:", (e as Error).message);
  }
}

async function run(): Promise<void> {
  const today = new Date().toISOString().split("T")[0];
  (document.getElementById("date") as HTMLInputElement).value = today;

  // Initialize WASM
  await loadWasm();
  wasmReady = true;

  document.getElementById("calcBtn")!.addEventListener("click", calculateZmanim);
  document.getElementById("locationBtn")!.addEventListener("click", handleGeolocation);

  setupAutocomplete();

  // Load data files in parallel
  await Promise.all([loadAdmin1(), loadCities()]);
}

run();
