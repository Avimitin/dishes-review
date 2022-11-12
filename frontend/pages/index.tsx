import { useRestaurants } from '../api'
import '../styles/Home.module.scss'

export default function Home() {
  return (
    <>
      <h1>小众点评</h1>
      <RestaurantList />
    </>
  )
}

function RestaurantList() {
  const restaurants = useRestaurants();
  if (restaurants.isLoading) {
    return (<div className="loading"><p>Loading...</p></div>)
  }
  if (restaurants.isError) {
    console.log(restaurants.isError);
    return (<div className="error"><p>Fail to fetch restaurant</p></div>)
  }

  const list = restaurants.data?.map((rest) => <li key={rest.id}>{`${rest.name} ${rest.address}`}</li>)

  return (<ul>
    {list}
  </ul>)
}
