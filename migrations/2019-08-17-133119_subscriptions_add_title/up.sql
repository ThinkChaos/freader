DROP TABLE IF EXISTS _subscriptions_old;

ALTER TABLE subscriptions
RENAME TO _subscriptions_old;

CREATE TABLE subscriptions (
  id BLOB PRIMARY KEY NOT NULL,
  feed_url VARCHAR(1024) NOT NULL,
  title VARCHAR(256) NOT NULL,

  CONSTRAINT unique_feed_url UNIQUE (feed_url)
);

INSERT INTO subscriptions
SELECT *, feed_url AS title FROM _subscriptions_old;
