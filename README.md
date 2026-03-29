## Structure

The _server_ handles login, socials and loading projects. One _worker_ is loaded per open project into a sandbox and handles file CRUD, building, and git.

## Building for production

- build frontend: `bun run build`
- build server:
  ```sh
  docker build -t dilladocs-server .
  docker run --rm dilladocs-server
  ```
- Add something akin to nginx.prod.conf to your nginx config, copy the frontend.

## Design goals

- Keep the server minimal, i.e. be resource efficient on idle
- avoid gross misuse

## Major TODOs

- Sandbox compilations
- easier file names
- Automatically close open files once no-one listens
- sync file index
- Implement TODOs & cleanup code

## Roadmap

- Git integration
- Cleanup frontend once I know enough about vue data flow and css layouting
- Improve sandboxing: Change the server architecture in a way where most code (yrs, project file logic) runs on a per-project worker.
- Editor WebRTC support

## Known bugs

- lange dateien kann man nicht zu ende scrollen

# jnr sagt:

- safari-quirk, auto pdf viewer doesn't show latest doc
- dateien umbennen
- LaTeX highlights so nicht hilfreich
