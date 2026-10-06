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

To check the queries at compile time we, need a database instance at wherever `database` in `textury.toml` is configured (usually `./db.sqlite`). To initialize the db run `cargo run --bin bootstrap_db`.

To start the server run `cargo run --bin server`. In a new terminal: run `pnpm dev` inside the `frontend` directory to start the UI and reverse proxy. The app should now be usable at `http://127.0.0.1:8001`.

Testing:
- `cargo test`
- To run integration test, run `pnpm exec playwright test` inside `frontend`

## Production setup

- build frontend: `pn build` or `pnpm build`
- Building the server with sandboxing needs [fireframe](https://codeberg.org/derdilla/fireframe) at `../fireframe` and this directory to be called `textury`
- To prepare the sandbox image run `./scripts/buildRootFS.sh`
  - To run fireframe:
    ```sh
    sudo ../fireframe/target/debug/fireframe \
      --jailer-path=<path_to_some>/firecracker/build/cargo_target/x86_64-unknown-linux-musl/release/jailer \
      --firecracker-path=<path_to_some>/firecracker/build/cargo_target/x86_64-unknown-linux-musl/release/firecracker \
      --kernel-path=<path_to_some>/vmlinux \
      --rootfs-path=./target/rootfs.ext4 \
      --chroot-root=<root_only_dir> \
      --self-socket-path="ff.sock" \
      --caller-gid=1000 # id of user running textury 
    ```
- build server:
  ```sh
  docker build -t textury-server -f textury/server/Dockerfile --build-context textury=./textury --build-context fireframe=./fireframe .
  docker volume create textury-data
  docker run  --security-opt unmask=ALL --rm -p 3000:3000 -v textury-data:/appdata --name textury -d textury-server
  # we need --security-opt unmask=ALL for basic bubblewrap sandboxing in the container. Probably doesn't apply when using fireframe
  ```
- Add something akin to `nginx.prod.conf` to your reverse proxy config; copy the frontend.

## License

This software is licensed as [EUPL version 1.2](https://eupl.eu/). You can find a copy at [LICENSE.txt](LICENSE.txt) and translated version at eupl.eu (e. g. [german](https://eupl.eu/1.2/de/)).

Please note, that you may not use Textury name and logo except for referring to this tool. We ask that you change the name, logo, and favicon of public deployments to avoid confusion. You should use the name, and at your own discretion logo, to link to this repository.
