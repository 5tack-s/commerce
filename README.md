# 🚧 UNDER CONSTRUCTION 🚧

Rust backend for [Scalable E-Commerce Platform](https://roadmap.sh/projects/scalable-ecommerce-platform)

This is mostly a learning exercise.

## Setup
1. Modify .env.example with desired values, then rename it to .env
2. Create a file named `db_password` containing the password to the database (commerce)
3. Start the database with `docker compose up`
4. Migrate:
```sh
$ DATABASE_URL=postgresql://<link-to-database> sea-orm-cli migrate fresh
```
5. Generate entities from the tables
```sh
$ sea-orm-cli generate entity -u postgresql://<link-to-database> -o src/entities
```

## To run:
```sh
$ docker compose up -d # Starts database
$ cargo run # Starts the backend
```

## Dispose later:
```sh
$ docker compose down -v # Erases volumes, in this case the database

$ docker compose down # Doesn't erase volumes
```

## Usage
We'll be using HTTPie.
```sh
$ http GET 127.0.0.1:3000/root
``` 

## Implemented features

### User service
- ✅ Registration
- ✅ Authentication
- ❌ Profile management

### Product service
- 🚧 Product listing
- 🚧 Categories

### Order service
- ❌ Placing orders
- ❌ Tracking orders
- ❌ Order history

### Shopping cart service
- 🚧 Adding/removing items
- 🚧 Updating quantities

### ~~Payment service~~
- ~~❌ Payment processing~~
- ~~❌ External payment integration~~

### Notification service
- ❌ Order confirmation
- ❌ Shipping updates

