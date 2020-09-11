CREATE TABLE subscriptions (
    id INTEGER PRIMARY KEY NOT NULL,
    feed_url VARCHAR(1024) NOT NULL,
    title VARCHAR(256) NOT NULL,
    site_url VARCHAR(256),
    next_refresh TIMESTAMP NOT NULL,
    error_count INTEGER NOT NULL,

    CONSTRAINT unique_feed_url UNIQUE (feed_url),
    CONSTRAINT positive_error_count CHECK(error_count >= 0)
);

CREATE TABLE categories (
    id INTEGER PRIMARY KEY NOT NULL,
    name VARCHAR(256) NOT NULL,

    CONSTRAINT unique_name UNIQUE (name)
);

CREATE TABLE subscription_categories (
    subscription_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,

    PRIMARY KEY(subscription_id, category_id),
    FOREIGN KEY(subscription_id) REFERENCES subscriptions(id),
    FOREIGN KEY(category_id) REFERENCES categories(id)
);
