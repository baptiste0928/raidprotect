# RaidProtect
This repository contain source code of the RaidProtect Discord bot and its API.

## Overview
The bot is actually split in three services :
- **Gateway**: connects to Discord, receive events and store cache. Events and cache are accessed with an internal communication based on a [remoc](https://docs.rs/remoc/) channel with TCP transport. An rate-limited HTTP proxy for the Discord API is also provided.
- **Event handler**: receive events from the gateway and handle them. Splitting the receiving part and the handling part allow for minor updates of the bot without restarting connection.
- **HTTP api**: public api used by the web dashboard to get information about guilds using the bot.

The dashboard can be considered as a separate service as it does not run in server-side.

<img src="https://user-images.githubusercontent.com/22115890/143934839-90148387-6bea-4802-b9e5-33b95a7ec8e8.png" width=60% height=60%>

## Crates hierarchy
The three services shares common libraries:
- **`model`**: contains the models used by the services (database, communication, ...).
- **`transport`**: servers and clients used to communicate between services.
- **`database`**: types and methods to interact with the database (MongoDB).
- **`util`**: various utility functions (logging, shutdown handling, ...).

Each service has a dedicated binary crate: `gateway`, `bot` and `api`.
The `workspace-hack` crate is used to improve build times and does not expose anything. It is managed by [cargo-hakari](https://lib.rs/crates/cargo-hakari).

<img src="https://user-images.githubusercontent.com/22115890/143934357-adc8146f-b2a7-41b9-8dc8-abfb962ad85b.png" width=60% height=60%>
