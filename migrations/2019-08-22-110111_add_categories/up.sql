CREATE TABLE categories (
  id BLOB PRIMARY KEY NOT NULL,
  name VARCHAR(256) NOT NULL,

  CONSTRAINT unique_name UNIQUE (name)
);

CREATE TABLE subscription_categories (
  subscription_id BLOB NOT NULL,
  category_id BLOB NOT NULL,

  PRIMARY KEY(subscription_id, category_id),
  FOREIGN KEY(subscription_id) REFERENCES subscriptions(id),
  FOREIGN KEY(category_id) REFERENCES categories(id)
);
