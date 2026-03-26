## Structure

The _server_ handles login, socials and loading projects. One _worker_ is loaded per open project into a sandbox and handles file CRUD, building, and git.

## Building for production

- build frontend: `bun run build`
- build server:
  ```sh
  docker build -t dilladocs-server .
  docker run --rm dilladocs-server
- Add something akin to nginx.prod.conf to your nginx config, copy the frontend.

## Design goals

- Keep the server minimal, i.e. be resource efficient on idle
- avoid gross misuse

## Major TODOs

- Automatically close and save open files
- LaTeX compile support
- Sandbox compilations
- Editor WebRTC support
- Improve project overview
- Compilation settings
- Implement TODOs

## Roadmap

- Git integration
- Cleanup frontend once I know enough about vue data flow and css layouting
- Improve sandboxing: Change the server architecture in a way where most code (yrs, project file logic) runs on a per-project worker.
