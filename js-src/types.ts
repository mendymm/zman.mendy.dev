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

export interface GeoLocationPosition {
  coords: {
    latitude: number;
    longitude: number;
  };
}

export interface GeoLocationError {
  code: number;
  message: string;
  PERMISSION_DENIED: number;
  POSITION_UNAVAILABLE: number;
  TIMEOUT: number;
}
