ALTER TABLE subscriptions
RENAME TO _subscriptions_old;

CREATE TABLE subscriptions (
  id CHAR(36) PRIMARY KEY NOT NULL,
  feed_url VARCHAR(1024) NOT NULL,
  title VARCHAR(256) NOT NULL,

  CONSTRAINT unique_feed_url UNIQUE (feed_url)
);

UPDATE subscriptions
SET title = feed_url;
