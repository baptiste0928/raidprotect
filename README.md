<div id="top"></div>

<div align="center">
  <img src="./.github/logo-heading.png" alt="RaidProtect" width="390" height="50">
  <br /><br />
  
  **Moderate your Discord server easily.**  
  [Getting started ➔](https://raidprotect.org/)  
  
 <br/>
  
 ![GitHub top language](https://img.shields.io/github/languages/top/raidprotect/raidprotect)
 ![GitHub commit activity](https://img.shields.io/github/commit-activity/m/raidprotect/raidprotect)
 [![Chat on Discord](https://img.shields.io/badge/discord-.gg%2Fraidprotect-5865F2?style=flat&logo=discord&logoColor=white)](https://discord.gg/raidprotect)
 [![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](.github/CODE_OF_CONDUCT.md)  
</div>

> ⚠ **Alpha version:** This version of RaidProtect is a complete rewrite and is still under development. It is not the currently available version when inviting the bot on your server. It is not ready for production, **do not use it for anything other than testing**.
> 
> We are open to contributions. Read the [contribution guidelines](CONTRIBUTING.md) to learn more.

<details>
  <summary>Table of contents</summary>
  
  1. [Overview](#overview)
  2. [Installation](#installation)
</details>


## Overview
RaidProtect is an open-source Discord bot focused on moderation and security. It offers many features, including:

- **Real-time moderation** such as anti-spam to prevent malicious users from harming your community by punishing them immediately without human intervention.
- **Protection against automated accounts** with active features like a captcha to verify each user that join your server.
- **Powerful moderation tools** to allow your moderators to manage punishements easily and keep track of each member.

RaidProtect is trusted by thousands of servers around the world. To add it to yours, follow the instructions on [our website](https://raidprotect.org).

### Community and support
Our community lives in our Discord server, we only use this repository to plan the bot development. For any question about the bot, you can join our server and contact our support team. We also provide a user documentation that explains how the bot works.

**[➡️ Read the user documentation](https://docs.raidprotect.org)**  
**[➡️ Join our Discord server](https://discord.gg/raidprotect)**

For more details, read the [`SUPPORT.md`](.github/support.md) file.

## Installation
RaidProtect is written in [Rust](https://www.rust-lang.org/) and uses the latest stable version of the compiler. It is designed to run on a Linux system, but should also work on Windows and MacOS (let us know if you have problems). It uses [MongoDB](https://www.mongodb.com/) as its database.

- **Open in GitPod (recommanded)**: the easiest way to launch RaidProtect is to use [GitPod](https://www.gitpod.io/), a cloud-based IDE. This allow you to have a ready-to-use environnement with everything installed to start developping on the bot. GitPod offers a generous free plan of 50 hours of usage per month. 

  [![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/baptiste0928/raidprotect)
- **Running locally**: make sure you have a MongoDB database installed to start the bot. A simple way is to use a Docker/Podman container to launch a local instance: `docker run --name mongodb-raidprotect -d -p 27017:27017 mongo:latest`.



<!--


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
-->
