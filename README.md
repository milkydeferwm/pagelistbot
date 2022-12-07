# Page List Bot #
[![Build Test](https://github.com/milkydeferwm/pagelistbot/actions/workflows/test.yml/badge.svg)](https://github.com/milkydeferwm/pagelistbot/actions/workflows/test.yml)

Re-implementation of the [legacy Page List Bot](https://github.com/milkydeferwm/pagelistbot-legacy), whose code is too tightly coupled to continue to work on.

The new Page List Bot is built as a frontend-backend application. To build the project, simply clone the repository and run:
```
cargo build --release --workspace
```
And find the compiled programs in `target/release`. Currently a nightly build of rustc is required. 

## Daemon ##
[![dependency status](https://deps.rs/repo/github/milkydeferwm/pagelistbot/status.svg?path=bin%2Fdaemon)](https://deps.rs/repo/github/milkydeferwm/pagelistbot?path=bin%2Fdaemon)

Daemon is the backend of Page List Bot. It does the actual bot work, parsing and querying and writing to pages. It is designed to handle multiple sites in a single process, and open a port to listen to commands.

The daemon executable is `pagelistbotd`. Source code is located in `bin/daemon`.

### Usage ###
```
pagelistbotd [-a <ADDR>] [-p <PORT>] [--startup <FILE>]
```
Available options:
<dl>
<dt><code>-a, --addr &lt;ADDR&gt;</code></dt>
<dd>The address to bind the server to. Defaults to <code>127.0.0.1</code> if omitted.</dd>
<dt><code>-p, --port &lt;PORT&gt;</code></dt>
<dd>The port server is listening to. Defaults to <code>7378</code> if omitted.</dd>
<dt><code>--startup &lt;FILE&gt;</code></dt>
<dd>Automatically reads a startup file when the daemon starts. <a href="#startup-file">See below</a> for the startup file specification.</dd>
</dl>

#### Startup File ####
<details><summary>Click to show specification</summary>

<p>

The daemon can optionally read a startup file when it starts. This is useful when the daemon is automatically killed and restarted (by k8s, for example), so the interrupted hosts will automatically resume running.</p>

The startup file is a JSON file, structured like below:
```
{
    "login": {
        "example": {
            "username": "Example",
            "password": "Yipee~!"
        },
        "another": {
            "username": "AnotherUser",
            "password": "AnotherPassword"
        }
    },
    "sites": {
        "enwiki": {
            "login": "example",
            "api": "https://en.wikipedia.org/w/api.php",
            "on_site_config": "User:Example",
            "prefer_bot_edit": false,
            "db_name": "enwiki_p"
        },
        "meta": {
            "login": "example",
            "api_endpoint": "https://meta.wikimedia.org/w/api.php",
            "prefer_bot_edit": true,
            "onsite": "User:Example"
        }
    }
}
```

The config file is split into two parts: `login` and `sites`.

##### Login Part #####
`login` stores all the user credentials used during startup. It consists of a `username` field and a `password` field.
<dl>
<dt><code>username</code></dt>
<dd>The username of this account.</dd>
<dt><code>password</code></dt>
<dd>The bot password for this account.</dd>
</dl>

##### Sites Part #####
`sites` stores all host configurations. There are five supported fields: `login`, `api`, `on_site_config`, `prefer_bot_edit` and `db_name`. The latter two are optional.
<dl>
<dt><code>login</code></dt>
<dd>This field refers to one of the user credential stored in the above <a href="#login-part">login part</a>. The referred credential must exist in that part, otherwise the daemon cannot start the host.</dd>
<dt><code>api</code></dt>
<dd>The API endpoint address of the target MediaWiki site. This should always end in <code>api.php</code>.</dd>
<dt><code>on_site_config</code></dt>
<dd>The location of the detailed host configuration. The "on site" configuration is stored as a JSON page on the website, put the page's title to this field.</dd>
<dt><code>prefer_bot_edit</code> (optional)</dt>
<dd>If the user account used by daemon has "<code>bot</code>" flag, Page List Bot can mark all its edits as a "bot edit", if this field is set to <code>true</code>. If this field is set to <code>false</code>, or the account does not have the "<code>bot</code>" flag, edits will not be marked "bot edit". If omitted, this field defaults to <code>false</code>.</dd>
<dt><code>db_name</code> (optional)</dt>
<dd><p>If the database is accessible, put the database name here. Page List Bot will try to execute all queried through direct database query, rather than calling API. If omitted, Page List Bot will call the API to solve the query.</p><!--
--><p>The ability to solve through direct database query is not yet implemented, so in fact this field is currently totally ignored.</p></dd>
</dl>
</details>

## Command Line Client ##
[![dependency status](https://deps.rs/repo/github/milkydeferwm/pagelistbot/status.svg?path=bin%2Fcli)](https://deps.rs/repo/github/milkydeferwm/pagelistbot?path=bin%2Fcli)

The command line client connects to the daemon through an HTTP JSONRPC client. You can use this frontend client to issue commands to the backend daemon.

The client executable is `pagelistbot`. Source code is located in `bin/cli`.

### Usage ###
```
pagelistbot [-a <ADDR>] [-p <PORT>] <COMMAND> <COMMAND ARGS>
```
Detailed documentation coming soon.
