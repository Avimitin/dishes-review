import React from "react";
import ReactDOM from "react-dom/client";
import Root from "./routes/root";
import "./index.css";
import { createBrowserRouter, RouterProvider, Route } from "react-router-dom";
import RestaurantDetail from "./routes/restaurants";
import Review from "./routes/reviews";

const router = createBrowserRouter([
  {
    path: "/",
    element: <Root />,
  },
  {
    path: "/restaurants/:id",
    element: <RestaurantDetail />,
  },
  {
    path: "/reviews/:id",
    element: <Review />
  },
]);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>
);
