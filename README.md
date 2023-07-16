# Page List Bot #
[![Build Test](https://github.com/milkydeferwm/pagelistbot/actions/workflows/test.yml/badge.svg)](https://github.com/milkydeferwm/pagelistbot/actions/workflows/test.yml)

Re-implementation of the [legacy Page List Bot](https://github.com/milkydeferwm/pagelistbot-legacy), whose code is too tightly coupled to continue to work on.

The new Page List Bot is built as a frontend-backend application. To build the project, simply clone the repository and run:
```
cargo build --release --workspace
```
And find the compiled programs in `target/release`. Currently a nightly build of rustc is required. 

## Query ##
[![dependency status](https://deps.rs/repo/github/milkydeferwm/pagelistbot/status.svg?path=bin%2Fquery)](https://deps.rs/repo/github/milkydeferwm/pagelistbot?path=bin%2Fquery)

Query is the actual program that interprets the query, and executing the query. It can be called directly, but you may want to let other programs call it.

### Usage ###
```
query [--site <SITE>] [--query <QUERY>] [--user <USER>] [--password <PASSWORD>] [--timeout <TIMEOUT>] [--limit <LIMIT>] [--json]
```
Available options:
<dl>
<dt><code>-s, --site &lt;SITE&gt;</code></dt>
<dd>The URL of the remote MediaWiki installation's <code>api.php</code>. You can visit <code>Special:Version</code> to find that piece of information. eg. <code>https://en.wikipedia.org/w/api.php</code>.</dd>
<dt><code>-q, --query &lt;QUERY&gt;</code></dt>
<dd>The query in string. Note you may want to escape certain characters. eg. <code>linkto(\"Main Page\")</code>.</dd>
<dt><code>-u, --user &lt;USER&gt;</code></dt>
<dd>The username to execute the query as. If the username is an empty string, the program will execute the query as an anonymous user. Defaults to an empty string.</dd>
<dt><code>-p, --password &lt;PASSWORD&gt;</code></dt>
<dd>The password of the corresponding username. The password is a bot password. To obtain a bot password for your account, go to <code>Special:BotPasswords</code>. If the username is empty, this will be ignored. Defaults to an empty string.</dd>
<dt><code>-t, --timeout &lt;TIMEOUT&gt;</code></dt>
<dd>The longest time in seconds to wait for results. After the specified time has elapsed, a warning will be emitted. Defaults to <code>120</code>, which equals to two minutes.</dd>
<dt><code>-l, --limit &lt;LIMIT&gt;</code></dt>
<dd>The maximum amount of results per query step. The total amount of results emitted is usually less than that number. The limit can also be overrided in the query. If the limit is exceeded, a warning will be emitted. Defaults to <code>10000</code>.</dd>
<dt><code>--json</code></dt>
<dd>Whether to print the results in JSON format. This is good for piping, but not good for human. Specify this flag to enable JSON outputing.</dd>
</dl>
