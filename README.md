> [!WARNING]
> This is still in early development. Breaking changes and bugs are to be expected on every commit.
> 
> It is functional for everyday use – if you are fine with getting your hands dirty.

# Textury
_the collaborative document forge for small deployments_

- LaTeX and Typst support
- Easy to use
- Resource efficient to self host
- Provides modern features (WIP)

## Development

Copy `textury.sample.toml` to `textury.toml`.

To start the server run `cargo run --bin server`. In a new terminal: run `pnpm dev` inside the `frontend` directory to start the UI and reverse proxy. The app should now be usable at `http://127.0.0.1:8001`.

### Testing
- `cargo test`
- To run integration test, run `pnpm exec playwright test` inside `frontend`

### SQLx

For compilation to "just work", sqlx needs information about checked queries. So whenever you change any `query!`, you need to run `DATABASE_URL="sqlite://./db.sqlite?mode=rwc" cargo sqlx prepare` in the `server` directory. 

If for some reason these files are not up-to-date, comment out the entirety of `main.rs` and `lib.rs` and run `cargo run --bin bootstrap_db` to generate `./db.sqlite`. You can than proceed as described above to update the `.sqlx` directory.

Further reading:
- https://docs.rs/sqlx/latest/sqlx/macro.query.html#offline-mode
- https://github.com/transact-rs/sqlx/blob/main/FAQ.md#how-do-i-compile-with-the-macros-without-needing-a-database-eg-in-ci

## Production setup

- build frontend: `pn build` or `pnpm build`
- Building the server with sandboxing 
- build server:
  ```sh
  docker build -t textury-server -f textury/server/Dockerfile --build-context textury=./textury
  docker volume create textury-data
  docker run  --security-opt unmask=ALL --rm -p 3000:3000 -v textury-data:/appdata --name textury -d textury-server
  # we need --security-opt unmask=ALL for basic bubblewrap sandboxing in the container.
  ```
- Add something akin to `nginx.prod.conf` to your reverse proxy config; copy the frontend.

- Alternative with docker compose:
```docker compose up --build```
Then available at http://localhost:8001.

## License

This software is licensed as [EUPL version 1.2](https://eupl.eu/). You can find a copy at [LICENSE.txt](LICENSE.txt) and translated version at eupl.eu (e. g. [german](https://eupl.eu/1.2/de/)).

Please note, that you may not use Textury name and logo except for referring to this tool. We ask that you change the name, logo, and favicon of public deployments to avoid confusion. You should use the name, and at your own discretion logo, to link to this repository.
