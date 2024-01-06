# API Daemon #
[![dependency status](https://deps.rs/repo/github/milkydeferwm/pagelistbot/status.svg?path=bin%2Fapi_daemon)](https://deps.rs/repo/github/milkydeferwm/pagelistbot?path=bin%2Fapi_daemon)

API Daemon provides API connection access between the program and the MediaWiki instance. All programs should access the functionality of the website through the daemon. Daemon is also responsible for periodically refreshing the connection.

## Usage ##
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

## Configuration File for API Daemon ##
API Daemon reads the following keys from the [configuration file](/README.md#configuration-file).
<dl>
<dt><code>username</code></dt>
<dd>The login username.</dd>
<dt><code>password</code></dt>
<dd>The login bot password. Bot passwords are different from regular login passwords. You should create and manage your bot passwords at "Special:BotPasswords".</dd>
<dt><code>api</code></dt>
<dd>The remote URL of "api.php". For example, in English Wikipedia, the URL of "api.php" is <code>https://en.wikipedia.org/w/api.php</code>. If you are not sure where it is, refer to "Special:Version" on your site.</dd>
</dl>

## Notes ##
When setting up a connection, API Daemon makes the following API calls in order:
1. Login.
2. Check for user right flags (<code>action=query&meta=userinfo&uiprop=rights</code>). Namely, the daemon checks for `apihighlimits` flag to determine the chunk size when making queries. The daemon also checks for `bot` flag to determine whether to mark any edits as bot edits.
3. Site information retrieval (<code>action=query&meta=siteinfo&siprop=general|namespaces|namespacealiases|interwikimap</code>). The returned value for this call can be used to build a [`TitleCodec`](https://docs.rs/mwtitle/latest/mwtitle/struct.TitleCodec.html) object.

Every hour, API Daemon reloads the configuration file. During this process, it:
1. Deprecates the existing connection and replaces it with a fresh connection. Connections need to be refreshed because in lieu of persisting forever, each login session only lives for a short amount of time.
2. If a connection no longer exists in the configuration file, the connection is dropped without new connections being set up. A connection is also dropped if the daemon cannot establish a fresh connection to replace the existing one.
3. If a new site configuration appears in the file, the connection is established and available for API call delegation.
