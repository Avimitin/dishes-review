import { useParams, Link } from "react-router-dom";
import { useBackend } from "../api";

interface Dish {
  id: number;
  rid: number;
  name: string;
  image: string | null;
}

export default function RestaurantDetail() {
  const { id } = useParams();
  if (!id) {
    return (
      <div>
        <h1>404 Page Not Found</h1>
      </div>
    );
  }
  const detail = useBackend<Dish[]>(`/api/v1/restaurants/${id}`);
  if (detail.isLoading) {
    return (
      <div>
        <h2>Loading...</h2>
      </div>
    );
  }
  if (detail.isError) {
    return (
      <div>
        <h2>Fail to load details</h2>
      </div>
    );
  }
  if (!detail.result) {
    console.error("internal error");
    return <></>;
  }
  const data = detail.result;
  return (
    <div>
      <ul>
        {data.map((d) => (
          <Dish key={d.id} dish={d} />
        ))}
      </ul>
    </div>
  );
}

function Dish({ dish }: { dish: Dish }) {
  return (
    <div>
      <li>
        <Link to={`/reviews/${dish.id}`}>
          <h1>{dish.name}</h1>
        </Link>
        {dish.image ? <img src={`${dish.image}`}></img> : <></>}
      </li>
    </div>
  );
}
