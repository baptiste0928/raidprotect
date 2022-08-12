# Contribution guidelines

Thank you for contributing to RaidProtect! This document explains how to
contribute in a meaningful way. **Please read it before making any issue or pull
request.**

We strongly recommend that you join our Discord server to be able to communicate
directly with other contributors. If you have never contributed to an
open-source project, you may want to read
[How to Contribute to Open Source](https://opensource.guide/how-to-contribute/).


**[➡️ Join our Discord server](https://discord.gg/raidprotect)**

## What you can contribute to

Contributing does not only mean contributing code. Although some programming
skills will be useful, you do not need to be a Rust developer to help.

For example, you can:
- Suggest new features and improvements in our Discord server.
- Run latest versions locally to ensure they work properly and give feedback on
  new features.
- Translate the bot or the user documentation in other languages.
- Help users by answering questions in our Discord server or improve the user
  documentation.
- Improve existing code, review pull request or implement new features.

If there is something you would like to help with and you are not sure if it is
possible, feel free to ask in our Discord server.

Issues that may be a good choice for a first contribution are labeled with
*good first issue*.

**[✨ Good first issues](https://github.com/raidprotect/raidprotect/contribute)**

## Issues and feature suggestions

Unlike some open-source projects, we target users who mostly do not use GitHub.
**Issues are only used to plan development, not to suggest new features or ask
help.** To request help, use our Discord server. We also have a website
dedicated to feature suggestions.

**[➡️ Suggest new features](https://feedback.raidprotect.org)**

However, you can use the issues to:
- Report bugs with precise information about them, such as logs. If you can't
reproduce the bug locally, report it to us on our Discord server instead.
- Suggest internal improvements on the bot. For end-user features, use the
dedicated website above.

Do not hesitate to give your feedback on existing issues and to ask to help.
**If you want to work on an issue**, let us know so we can assign it to you.
This will avoid having two people working on the same thing.

## Pull requests

To make changes to the code, you must create a pull request. If you have never
created one yet, you can read
[First Contributions](https://github.com/firstcontributions/first-contributions).

Before creating a PR, **please open an issue** (see the previous section) and
ask to work on it. This will ensure that what you are working on will not be
rejected. However, you can create a PR directly if you made only small changes,
such as a minor bug fix or documentation improvements.

When modifying the code, try to reproduce as much as possible the style of the
existing code. For example, we usually use `thiserror` for error types, so do
the same even if it is not in your habits.

Once your PR is submitted, a contributor will review it as soon as possible. To
make the job easier, **ensure that all automated tests pass** before submitting.
Specifically, check the following:

- The code must compile and pass `cargo clippy` without any error or warnings.
- Ensure the code is properly formatted with `cargo fmt`.
- Document all the functions/types you create, and check that `cargo doc`
  does not produce any errors or warnings.
- Check the spelling of comments and variable names with
  [`codespell`](https://github.com/codespell-project/codespell).

Pull request titles should follow the
[conventional commits](https://www.conventionalcommits.org/en/v1.0.0/) format.

Once your PR has been opened, please remain available if we need to ask for
changes. Thank you for your contribution! 🎉

## Localization

This project rely on Localazy to manage translations. If you want to help,
join the [project on Localazy](https://localazy.com/p/raidprotect-bot).

Since the maintainers are not native English speakers, the fallback languages
is set to French, which means that all strings must be translated in French for
the code to compile. If you want to do a contribution which implies **adding a
localized string**, use a web translator for the French translation and let us
know in the PR. We will review the translation and fix it if needed.
