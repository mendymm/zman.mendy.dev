export interface GeolocationResult {
	lat: number;
	lon: number;
}

export function getCurrentPosition(): Promise<GeolocationResult> {
	return new Promise((resolve, reject) => {
		if (!navigator.geolocation) {
			reject(new Error('Geolocation is not supported by your browser'));
			return;
		}
		navigator.geolocation.getCurrentPosition(
			(position) => {
				resolve({ lat: position.coords.latitude, lon: position.coords.longitude });
			},
			(error) => {
				let message = 'Unable to get your location';
				switch (error.code) {
					case error.PERMISSION_DENIED:
						message = 'Location permission denied. Please enable location access.';
						break;
					case error.POSITION_UNAVAILABLE:
						message = 'Location information unavailable';
						break;
					case error.TIMEOUT:
						message = 'Location request timed out';
						break;
				}
				reject(new Error(message));
			},
			{ enableHighAccuracy: true, timeout: 10000, maximumAge: 0 }
		);
	});
}
