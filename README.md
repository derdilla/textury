## Structure

The _server_ handles login, socials and loading projects. One _worker_ is loaded per open project into a sandbox and handles file CRUD, building, and git.

## Bindings

To generate data types for the client, run `cargo test` in `./server`.

## Building for production

- build frontend: `pn build` or `pnpm build`
- build server:
  ```sh
  docker build -t dilladocs-server 
  docker volume create dilladocs-data
  docker run --rm -p 3000:3000 -v dilladocs-data:/appdata --name dilladocs -d dilladocs-server
  ```
- Add something akin to nginx.prod.conf to your nginx config, copy the frontend.

## Design goals

- Keep the server minimal, i.e. be resource efficient on idle
- avoid gross misuse

## Major TODOs

- Implement TODOs & cleanup code
  - stricter lint set
