import { useParams } from "react-router-dom";
import { useBackend } from "../api";

interface Review {
  reviewer: number;
  score: number;
  details: string;
}

export default function Review() {
  const { id } = useParams();
  if (!id) {
    throw Error("Page not found");
  }

  const response = useBackend<Review>(`/api/v1/dishes/${id}`);
  if (response.isLoading) {
    return (
      <div>
        <h2>Loading...</h2>
      </div>
    );
  }
  if (response.isError) {
    return (
      <div>
        <h2>Fail to load details</h2>
      </div>
    );
  }
  if (!response.result) {
    throw new Error()
  }

  return <div>
    <h2>评分</h2><div>{response.result.score}</div>
    <p>{response.result.details}</p>
  </div>
}
