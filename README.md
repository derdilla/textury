## Structure

The _server_ handles login, socials and loading projects. One _worker_ is loaded per open project into a sandbox and handles file CRUD, building, and git.

## Bindings

To generate data types for the client, run `cargo test` in `./server`.

## Development

To check the queries at compile time we, need a database instance at whatever the `DATABASE_URL` environment variable is (usually `./db.sqlite`). To initialize the db in order to compile the server run `cargo run --bin bootstrap_db`.

## Building for production

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
  docker build -t textury-server 
  docker volume create textury-data
  docker run --rm -p 3000:3000 -v textury-data:/appdata --name textury -d textury-server
  ```
- Add something akin to nginx.prod.conf to your nginx config, copy the frontend.

## Design goals

- Keep the server minimal, i.e. be resource efficient on idle
- avoid gross misuse

## Major TODOs

- Implement TODOs & cleanup code
  - stricter lint set
