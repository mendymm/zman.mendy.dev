import json
import requests
import time
import os

INPUT_FILE = "data/all-cities.jsonl"
OUTPUT_FILE = "data/coords_to_elevation.json"
BATCH_SIZE = 100
API_URL = "https://api.opentopodata.org/v1/aster30m"
MAX_RETRIES = 5


def load_results():
    if os.path.exists(OUTPUT_FILE):
        with open(OUTPUT_FILE, "r") as f:
            return json.load(f)
    return {}


def save_results(results):
    with open(OUTPUT_FILE, "w") as f:
        json.dump(results, f, indent=4)


def process_batch(batch_coords):
    locations_str = "|".join([f"{lat},{lng}" for lat, lng in batch_coords])

    for attempt in range(MAX_RETRIES):
        response = requests.post(
            API_URL,
            json={"locations": locations_str},
        )

        if response.status_code == 429:
            wait_time = 60 * (attempt + 1)
            print(
                f"Rate limited. Waiting {wait_time}s before retry {attempt + 1}/{MAX_RETRIES}"
            )
            time.sleep(wait_time)
            continue

        response.raise_for_status()
        return response.json()["results"]

    raise Exception(f"Max retries ({MAX_RETRIES}) exceeded for batch")


def main():
    total_lines = 167603
    results = load_results()
    already_count = len(results)

    print(f"Loaded {already_count} already processed coordinates")

    with open(INPUT_FILE, "r") as infile:
        batch_coords = []
        for line_num, line in enumerate(infile, 1):
            data = json.loads(line)
            coords = data["coordinates"]
            lat = coords["lat"]
            lng = coords["lon"]
            coord_key = f"{lat},{lng}"

            if coord_key in results:
                continue

            batch_coords.append((lat, lng, coord_key))

            if len(batch_coords) == BATCH_SIZE:
                api_results = process_batch(
                    [(lat, lng) for lat, lng, _ in batch_coords]
                )

                for (_, _, coord_key), api_result in zip(batch_coords, api_results):
                    results[coord_key] = api_result["elevation"]

                save_results(results)

                print(
                    f"Processed {len(results)}/{total_lines} lines ({len(results) / total_lines * 100:.2f}%)"
                )
                batch_coords = []

        if batch_coords:
            api_results = process_batch([(lat, lng) for lat, lng, _ in batch_coords])

            for (_, _, coord_key), api_result in zip(batch_coords, api_results):
                results[coord_key] = api_result["elevation"]

            save_results(results)

            print(
                f"Processed {len(results)}/{total_lines} lines ({len(results) / total_lines * 100:.2f}%)"
            )

    print("Done!")


if __name__ == "__main__":
    main()
