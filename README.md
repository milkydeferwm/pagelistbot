# Page List Bot #
[![Build Test](https://github.com/milkydeferwm/pagelistbot/actions/workflows/test.yml/badge.svg)](https://github.com/milkydeferwm/pagelistbot/actions/workflows/test.yml)

Re-implementation of the [legacy Page List Bot](https://github.com/milkydeferwm/pagelistbot-legacy), whose code is too tightly coupled to continue to work on.

The new Page List Bot is built with frontend-backend in mind. You can find backend code in `bin/daemon`.

More documentation coming soon.

## Daemon ##
[![dependency status](https://deps.rs/repo/github/milkydeferwm/pagelistbot/status.svg?path=bin%2Fdaemon)](https://deps.rs/repo/github/milkydeferwm/pagelistbot?path=bin%2Fdaemon)

Daemon is the backend of Page List Bot. It does the actual bot work, parsing and querying and writing to pages. It is designed to handle multiple sites in a single process, and open a port to listen to commands.
