-- Add migration script here
CREATE TABLE IF NOT EXISTS reviewer (
  id   INT,
  name TEXT,
  UNIQUE(id)
);

CREATE TABLE IF NOT EXISTS restaurant (
  id      INTEGER PRIMARY KEY AUTOINCREMENT,
  name    TEXT,
  address TEXT
);

CREATE TABLE IF NOT EXISTS review (
  id       INTEGER PRIMARY KEY AUTOINCREMENT,
  reviewer INT,
  dish     INT,
  details  TEXT,
  score    INT,
  FOREIGN KEY(reviewer) REFERENCES reviewer(id),
  FOREIGN KEY(dish) REFERENCES dish(id)
);

CREATE TABLE IF NOT EXISTS dish (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  restaurant INT,
  name       TEXT,
  image      TEXT,
  FOREIGN KEY(restaurant) REFERENCES restaurant(id)
);
