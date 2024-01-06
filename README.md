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
  <thead>
    <tr>
      <th scope="col">Component</th>
      <th scope="col">Description</th>
      <th scope="col">Badge</th>
    </tr>
  </thead>
  <thead>
    <tr>
      <th colspan="3">Executable</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <th scope="row"><a href="/bin/api_daemon/">api_daemon</a></th>
      <td>Proxy for all connections to MediaWiki API.</td>
      <td><a href="https://deps.rs/repo/github/milkydeferwm/pagelistbot?path=bin%2Fapi_daemon"><img src="https://deps.rs/repo/github/milkydeferwm/pagelistbot/status.svg?path=bin%2Fapi_daemon"/></a></td>
    </tr>
    <tr>
      <th scope="row"><a href="/bin/query/">query</a></th>
      <td>Query executor.</td>
      <td><a href="https://deps.rs/repo/github/milkydeferwm/pagelistbot?path=bin%2Fquery"><img src="https://deps.rs/repo/github/milkydeferwm/pagelistbot/status.svg?path=bin%2Fquery"/></a></td>
    </tr>
  </tbody>
  <thead>
    <tr>
      <th colspan="3">Library</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <th scope="row"><a href="/lib/api_daemon_interface/">api_daemon_interface</a></th>
      <td>JSON RPC interface definitions shared between the API Daemon and other programs.</td>
      <td><a href="https://deps.rs/repo/github/milkydeferwm/pagelistbot?path=lib%2Fapi-daemon-interface"><img src="https://deps.rs/repo/github/milkydeferwm/pagelistbot/status.svg?path=lib%2Fapi_daemon_interface"/></a></td>
    </tr>
    <tr>
      <th scope="row"><a href="/lib/ast/">ast</a></th>
      <td>Abstract syntax tree (AST) for the query language.</td>
      <td><a href="https://deps.rs/repo/github/milkydeferwm/pagelistbot?path=lib%2Fast"><img src="https://deps.rs/repo/github/milkydeferwm/pagelistbot/status.svg?path=lib%2Fast"/></a></td>
    </tr>
    <tr>
      <th scope="row"><a href="/lib/env/">env</a></th>
      <td>Environment variable utilities.</td>
      <td><a href="https://deps.rs/repo/github/milkydeferwm/pagelistbot?path=lib%2Fenv"><img src="https://deps.rs/repo/github/milkydeferwm/pagelistbot/status.svg?path=lib%2Fenv"/></a></td>
    </tr>
    <tr>
      <th scope="row"><a href="/lib/intorinf/">intorinf</a></th>
      <td>A special integer type holding either a finite integer value or an infinite value.</td>
      <td><a href="https://deps.rs/repo/github/milkydeferwm/pagelistbot?path=lib%2Fintorinf"><img src="https://deps.rs/repo/github/milkydeferwm/pagelistbot/status.svg?path=lib%2Fintorinf"/></a></td>
    </tr>
    <tr>
      <th scope="row"><a href="/lib/provider/">provider</a></th>
      <td>Trait definition for data provider.</td>
      <td><a href="https://deps.rs/repo/github/milkydeferwm/pagelistbot?path=lib%2Fprovider"><img src="https://deps.rs/repo/github/milkydeferwm/pagelistbot/status.svg?path=lib%2Fprovider"/></a></td>
    </tr>
    <tr>
      <th scope="row"><a href="/lib/solver/">solver</a></th>
      <td>Query solver. Each query opeartion is defined as a poll-able stream.</td>
      <td><a href="https://deps.rs/repo/github/milkydeferwm/pagelistbot?path=lib%2Fsolver"><img src="https://deps.rs/repo/github/milkydeferwm/pagelistbot/status.svg?path=lib%2Fsolver"/></a></td>
    </tr>
    <tr>
      <th scope="row"><a href="/lib/trioresult/">trioresult</a></th>
      <td>A tri-way result type. Can hold a value, or a warning, or an error.</td>
      <td><a href="https://deps.rs/repo/github/milkydeferwm/pagelistbot?path=lib%2Ftrioresult"><img src="https://deps.rs/repo/github/milkydeferwm/pagelistbot/status.svg?path=lib%2Ftrioresult"/></a></td>
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
