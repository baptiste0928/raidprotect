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

> ⚠ **Alpha version:** This version of RaidProtect is a complete rewrite and is
> still under development. It is not the currently available version when
> inviting the bot on your server. It is not ready for production, **do not use
> it for anything other than testing**.
> 
> We are open to contributions. Read the [contribution guidelines](CONTRIBUTING.md)
> to learn more.

<details>
  <summary>Table of contents</summary>
  
  1. [Overview](#overview-)
  2. [Installation](#installation)
  3. [Contributing](#contributing)
  4. [License](#license)
</details>


## Overview ✨
RaidProtect is an open-source Discord bot focused on moderation and security. It
offers many features, including:

- **Real-time moderation** such as anti-spam to prevent malicious users from
  harming your community by punishing them immediately without human intervention.
- **Protection against automated accounts** with active features like a captcha 
  to verify each user that join your server.
- **Powerful moderation tools** to allow your moderators to manage punishments 
  easily and keep track of each member.

RaidProtect is trusted by thousands of servers around the world. To add it to
yours, follow the instructions on [our website](https://raidprotect.org).

### Community and support
Our community lives in our Discord server, we only use this repository to plan
the bot development. For any question about the bot, you can join our server and
contact our support team. We also provide a user documentation that explains how
the bot works.

**[➡️ Read the user documentation](https://docs.raidprotect.org)**  
**[➡️ Join our Discord server](https://discord.gg/raidprotect)**

For more details, read the [`SUPPORT.md`](.github/support.md) file.

## Installation
RaidProtect is written in [Rust](https://www.rust-lang.org/) and uses the latest
stable version of the compiler. It is designed to run on a Linux system, but
should also work on Windows and MacOS (let us know if you have problems). It
uses [MongoDB](https://www.mongodb.com/) as its database.

- **Open in GitPod (recommanded)**: the easiest way to launch RaidProtect is to
use [GitPod](https://www.gitpod.io/), a cloud-based IDE. This allow you to have
a ready-to-use environnement with everything installed to start developing on
the bot. GitPod offers a generous free plan of 50 hours of usage per month. 

  [![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/raidprotect/raidprotect)
  
- **Running locally**: make sure you have a MongoDB database installed to start
the bot. A simple way is to use a Docker/Podman container to launch a local
instance: `docker run --name mongodb-raidprotect -d -p 27017:27017 mongo:latest`.

### Creating the bot account
You must create a bot account in order to launch RaidProtect. See
[this page](https://discordjs.guide/preparations/setting-up-a-bot-application.html)
for more information on how to do this. You also need to enable the **server
member and message content intents** from the bot settings.

**[➡️ Discord Developer Portal](https://discord.com/developers/applications)**

Then, invite the bot account you created in at least one server to be able to
use it. RaidProtect requires the `ADMINISTRATOR` permission ad the
`applications.commands` scope. You can get an invite URL using the OAuth Url
Generator in the Discord Developer Portal.

### Basic configuration
RaidProtect load configuration from environement variables prefixed with `RAIDPROTECT_`.

- If you are using GitPod, you can set project-specific environment variables
  using command line or in your account settings
  ([instructions](https://www.gitpod.io/docs/environment-variables/#project-specific-environment-variables)). 
  This is the preferred way to persist variables between multiple workspace.
- If you develop locally, you can write your environement variables in a `.env`
  file in the project root. These variables will be loaded when the bot launches.

**The only required configuration is the bot token** with the `RAIDPROTECT_TOKEN`
environement variable. This token can be obtained from the Discord Developer
Portal. You may also set the `RAIDPROTECT_COMMAND_GUILD` with the id of the
server you use for testing the bot. This will register slash commands as guild
rather than global commands, so they will be updated instantly.

For a complete and up-to-date list of available configuration options, refer to
the [`raidprotect/src/config.rs`](raidprotect/src/config.rs) file.

### Starting the bot
You should be able to compile and launch the bot with `cargo run` (ensure the
MongoDB database is running and exposed to `localhost:27017` - this can be
changed in the configuration). Feel free to ask in our Discord server if you run
into any problem.

Congratulations, you now have a working local instance of RaidProtect. 🎉

## Contributing
RaidProtect is an open-source project and we are happy to welcome new contributors.
You can help in many ways, from improving functionality to fixing bugs. Feel
free to join [our Discord server](https://discord.com/raidprotect) to chat with
us, we will be happy to help you get started on the project. 

A good place to start is to look at the issues that are not yet assigned and ask
to do them. Don't forget to read the contribution guidelines first.

**[➡️ Contribution guidelines](CONTRIBUTING.md)**

## License
RaidProtect is licensed under the [GNU AGPL](LICENSE) license. This is a copyleft
license, which gives you the right to use, modify and redistribute RaidProtect
under the following conditions:

- The source code of any modified version of the bot (*fork*) **MUST** be easily
  published under the GNU AGPL license.
- Any bot including all of a part of the RaidProtect source code **MUST** 
  clearly state that it is derivated from RaidProtect.

The conditions below are a summary and have no legal value. The full license is
available in the [LICENSE](LICENSE) file.

> RaidProtect
> Copyright (C) 2022  RaidProtect Contributors
>
> This program is free software: you can redistribute it and/or modify
> it under the terms of the GNU Affero General Public License as published
> by the Free Software Foundation, either version 3 of the License, or
> (at your option) any later version.
>
> This program is distributed in the hope that it will be useful,
> but WITHOUT ANY WARRANTY; without even the implied warranty of
> MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
> GNU Affero General Public License for more details.
>
> You should have received a copy of the GNU Affero General Public License
> along with this program.  If not, see <https://www.gnu.org/licenses/>.

### RaidProtect trademark and logo
The use of the RaidProtect name and logo is allowed only for referring to this
project. This must not imply any official involvement without prior permission.
If in doubt, ask us before using the RaidProtect name and/or logo.

[Return to the top ⮝](#top)
