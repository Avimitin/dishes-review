import useSWR from "swr";
import config from "../config.json"

async function jsonFetcher<T>(url: URL): Promise<T> {
    const resp = await fetch(url);
    return await resp.json();
}

export interface Restaurant {
    id: number,
    name: string,
    address: string,
}

export function useRestaurants(id?: number) {
    const url = new URL("/api/v1/restaurants", config.backend.address);
    const { data, error } = useSWR<Restaurant[]>(url.href, jsonFetcher);

    return {
        isLoading: !data && !error,
        isError: error,
        result: data,
    }
}