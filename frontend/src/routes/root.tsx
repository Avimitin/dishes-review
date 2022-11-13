import { Restaurant, useRestaurants } from "../api";
import { Link } from "react-router-dom";

export default function Root() {
  return (
    <div>
      <h1>小众点评</h1>
      <RestaurantsList />
    </div>
  );
}

function RestaurantsList() {
  const resp = useRestaurants();
  if (resp.isLoading) {
    return (
      <div>
        <h2>Loading..</h2>
      </div>
    );
  }
  if (resp.isError) {
    console.error(resp.isError);
    return (
      <div>
        <p>Fail to fetch restaurants</p>
      </div>
    );
  }

  const list = resp.result?.map((res) => (
    <RestaurantUnit rest={res} key={res.id} />
  ));

  return (
    <div>
      <ul>{list}</ul>
    </div>
  );
}

function RestaurantUnit({ rest }: { rest: Restaurant }) {
  return (
    <div>
      <li>
        <Link to={`/restaurants/${rest.id}`}>
          {`${rest.name} ${rest.address}`}
        </Link>
      </li>
    </div>
  );
}
