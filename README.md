# Page List Bot #
[![Build Test](https://github.com/milkydeferwm/pagelistbot/actions/workflows/test.yml/badge.svg)](https://github.com/milkydeferwm/pagelistbot/actions/workflows/test.yml)

Re-implementation of the [legacy Page List Bot](https://github.com/milkydeferwm/pagelistbot-legacy), whose code is too tightly coupled to continue to work on.

The new Page List Bot is built as a frontend-backend application. To build the project, simply clone the repository and run:
```
cargo build --release --workspace
```
And find the compiled programs in `target/release`. Currently a nightly build of rustc is required. 

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

## API Daemon ##
[![dependency status](https://deps.rs/repo/github/milkydeferwm/pagelistbot/status.svg?path=bin%2Fapi_daemon)](https://deps.rs/repo/github/milkydeferwm/pagelistbot?path=bin%2Fapi_daemon)

API Daemon provides API connection access between the program and the MediaWiki instance. All programs should access the functionality of the website through the daemon. Daemon is also responsible for periodically refreshing the connection.

### Usage ###
```
api-daemon [--config <PATH>] [--bind-all] [--port <PORT>]
```
Available options:
<dl>
<dt><code>-c, --config &lt;PATH&gt;</code></dt>
<dd>The location of the configuration file. If not explicitly specified, the program tries to read from the default location <a href=#configuration-and-environment-variables>outlined above</a>.</dd>
<dt><code>--bind-all</code></dt>
<dd>If this flag is NOT set, the daemon process will only listen to <code>localhost</code>; if this flag is set, the daemon process will listen to <code>0.0.0.0</code> and accept requests from all addresses.</dd>
<dt><code>-p, --port &lt;PORT&gt;</code></dt>
<dd>The port this program listens to. Should be an integer between 0 and 65535 and should not clash with other processes. Defaults to <code>8848</code>.</dd>
</dl>

### Configuration File for API Daemon ###
API Daemon reads the following keys from the configuration file.
<dl>
<dt><code>username</code></dt>
<dd>The login username.</dd>
<dt><code>password</code></dt>
<dd>The login bot password. Bot passwords are different from regular login passwords. You should create and manage your bot passwords at "Special:BotPasswords".</dd>
<dt><code>api</code></dt>
<dd>The remote URL of "api.php". For example, in English Wikipedia, the URL of "api.php" is <code>https://en.wikipedia.org/w/api.php</code>. If you are not sure where it is, refer to "Special:Version" on your site.</dd>
</dl>
Every once per hour, API Daemon reloads the configuration file and add/remove/refresh the connections. Connections are dropped if they no longer exist in the configuration file, or if there are internet connection problems.

## Query ##
[![dependency status](https://deps.rs/repo/github/milkydeferwm/pagelistbot/status.svg?path=bin%2Fquery)](https://deps.rs/repo/github/milkydeferwm/pagelistbot?path=bin%2Fquery)

Query is the actual program that interprets the query, and executing the query. It can be called directly, but you may want to let other programs call it. Query requires an accessible [API Daemon](#api-daemon) to work properly.

### Usage ###
```
query [--addr <ADDR>] [--port <PORT>] [--key <KEY>] [--query <QUERY>] [--timeout <TIMEOUT>] [--limit <LIMIT>] [--json]
```
Available options:
<dl>
<dt><code>-a, --addr &lt;ADDR&gt;</code></dt>
<dd>The address of the API Daemon. Defaults to <code>127.0.0.1</code>, aka <code>localhost</code>.</dd>
<dt><code>-p, --port &lt;PORT&gt;</code></dt>
<dd>The port of the API Daemon. Defaults to <code>8848</code>.</dd>
<dt><code>-k, --key &lt;KEY&gt;</code></dt>
<dd>Use this option to specify which website the query is made against. For example, if this query is made against English Wikipedia, and in <code>config.toml</code>, the login information is stored under site key "enwiki", then type <code>enwiki</code>.</dd>
<dt><code>-q, --query &lt;QUERY&gt;</code></dt>
<dd>The query in string. Note you may want to escape certain characters. eg. <code>linkto(\"Main Page\")</code>.</dd>
<dt><code>-t, --timeout &lt;TIMEOUT&gt;</code></dt>
<dd>The longest time in seconds to wait for results. After the specified time has elapsed, a warning will be emitted. Defaults to <code>120</code>, which equals to two minutes.</dd>
<dt><code>-l, --limit &lt;LIMIT&gt;</code></dt>
<dd>The maximum amount of results per query step. The total amount of results emitted is usually less than that number. The limit can also be overrided in the query. If the limit is exceeded, a warning will be emitted. Defaults to <code>10000</code>.</dd>
<dt><code>--json</code></dt>
<dd>Whether to print the results in JSON format. This is good for piping, but not good for human. Specify this flag to enable JSON outputing.</dd>
</dl>
