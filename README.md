# Page List Bot #
[![Build Test](https://github.com/milkydeferwm/pagelistbot/actions/workflows/test.yml/badge.svg)](https://github.com/milkydeferwm/pagelistbot/actions/workflows/test.yml)

Re-implementation of the [legacy Page List Bot](https://github.com/milkydeferwm/pagelistbot-legacy), whose code is too tightly coupled to continue to work on.

The new Page List Bot is built as a frontend-backend application. To build the project, simply clone the repository and run:
```
cargo build --release --workspace
```
And find the compiled programs in `target/release`. Currently a nightly build of rustc is required. 

## Components ##
The main function of Page List Bot is split into several small parts listed below. Each component has its own documentation.

<table>
  <caption>Overview of components</caption>
  <thead>
    <tr>
      <th scope="col">Component</th>
      <th scope="col">Description</th>
      <th scope="col">Badge</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td colspan="3">Executable</td>
    </tr>
    <tr>
      <th scope="row">API Daemon</th>
      <td>Proxy for all connections to MediaWiki API.</td>
      <td><a href="https://deps.rs/repo/github/milkydeferwm/pagelistbot?path=bin%2Fapi_daemon"><img src="https://deps.rs/repo/github/milkydeferwm/pagelistbot/status.svg?path=bin%2Fapi_daemon"/></a></td>
    </tr>
    <tr>
      <th scope="row">Query</th>
      <td>Query executor.</td>
      <td><a href="https://deps.rs/repo/github/milkydeferwm/pagelistbot?path=bin%2Fquery"><img src="https://deps.rs/repo/github/milkydeferwm/pagelistbot/status.svg?path=bin%2Fquery"/></a></td>
    </tr>
    <tr>
      <td colspan="3">Library</td>
    </tr>
  </tbody>
</table>

## Configuration and Environment Variables ##
### Directory Hierarchy ###
The Page List Bot suite assumes the following directory hierarchy:
```
${PAGELISTBOT_HOME}
├─ bin
│   ├─ api-daemon
│   ├─ query
│   └─ ...
├─ logs
│   ├─ logxxxxxx.log
│   └─ ...
└─ config.toml
```
`PAGELISTBOT_HOME` is the environment variable used to locate the installation directory. If it is not specified, `~/.pagelistbot` is used.

### Configuration File ###
The configuration is stored in `config.toml`. It follows TOML syntax:
```toml
[site key 1]
key1 = value1
key2 = value2

[site key 2]
key1 = value1
key2 = value2

...
```
It is shared between programs, each reading different parts of the configuration.
> **Be careful.** Bot username and password may be saved into this file. This file should therefore be considered confidential.
