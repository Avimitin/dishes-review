import useSWR from "swr";
import config from "../config.json";

async function jsonFetcher<T>(url: URL): Promise<T> {
  const resp = await fetch(url);
  return await resp.json();
}

export function useBackend<T>(suffix: string) {
  const url = new URL(suffix, config.backend.address);
  const { data, error } = useSWR<T>(url.href, jsonFetcher);

  return {
    isLoading: !data && !error,
    isError: error,
    result: data,
  };
}