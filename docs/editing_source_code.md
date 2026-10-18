# Editing the code

We try to keep the codebase small and well organized, so hacking here can be fun!

1. Copy `textury.sample.toml` to `textury.toml`
2. `cargo run --bin server` starts the current code
3.  `pn dev` in `frontend` proxies the server and serves the UI with hot-reload to http://127.0.0.1:8001

To start the server run `cargo run --bin server`. In a new terminal: run `pnpm dev` inside the `frontend` directory to start the UI and reverse proxy. The app should now be usable at `http://127.0.0.1:8001`.

## Testing
- `cargo test`
- To run integration test, run `pnpm exec playwright test` inside `frontend`

## SQLx

For compilation to "just work", sqlx needs information about checked queries. So whenever you change any `query!`, you need to run `DATABASE_URL="sqlite://./db.sqlite?mode=rwc" cargo sqlx prepare` in the `server` directory. 

If for some reason these files are not up-to-date, comment out the entirety of `main.rs` and `lib.rs` and run `cargo run --bin bootstrap_db` to generate `./db.sqlite`. You can than proceed as described above to update the `.sqlx` directory.

Further reading:
- https://docs.rs/sqlx/latest/sqlx/macro.query.html#offline-mode
- https://github.com/transact-rs/sqlx/blob/main/FAQ.md#how-do-i-compile-with-the-macros-without-needing-a-database-eg-in-ci
