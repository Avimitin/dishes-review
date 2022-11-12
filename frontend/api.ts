import config from "./config.json";
import useSWR from "swr";

export interface Restaurants {
  id: number,
  name: string,
  address: string
}

async function jsonFetch<T>(url: RequestInfo | URL): Promise<T> {
  const res = await fetch(url);
  const object: T = await res.json();
  return object
}

export function useRestaurants(id?: number) {
  const { data, error } = useSWR<Restaurants[]>(`${config.backend.address}/api/v1/restaurants`, jsonFetch)

  return {
    isLoading: !data && !error,
    isError: error,
    data: data,
  }
}
