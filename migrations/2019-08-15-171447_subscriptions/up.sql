CREATE TABLE subscriptions (
  id CHAR(36) PRIMARY KEY NOT NULL,
  feed_url VARCHAR NOT NULL,

  CONSTRAINT unique_feed_url UNIQUE (feed_url)
);
