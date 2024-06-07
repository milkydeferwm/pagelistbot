# Query #
[![dependency status](https://deps.rs/repo/github/milkydeferwm/pagelistbot/status.svg?path=bin%2Fquery)](https://deps.rs/repo/github/milkydeferwm/pagelistbot?path=bin%2Fquery)

Query is the actual program that interprets the query, and executing the query. It can be called directly, but you may want to let other programs call it. Query requires an accessible [API Daemon](/bin/api_daemon/) to work properly.

## Usage ##
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
<dd>Use this option to specify which website the query is made against. For example, if this query is made against English Wikipedia, and in the configuration file, the login information is stored under site key "enwiki", then type <code>enwiki</code>.</dd>
<dt><code>-q, --query &lt;QUERY&gt;</code></dt>
<dd>The query in string. Note you may want to escape certain characters. eg. <code>linkto(\"Main Page\")</code>.</dd>
<dt><code>-t, --timeout &lt;TIMEOUT&gt;</code></dt>
<dd>The longest time in seconds to wait for results. After the specified time has elapsed, a warning will be emitted. Defaults to <code>120</code>, which equals to two minutes.</dd>
<dt><code>-l, --limit &lt;LIMIT&gt;</code></dt>
<dd>The maximum amount of results per query step. The total amount of results emitted is usually less than that number. The limit can also be overrided in the query. If the limit is exceeded, a warning will be emitted. Defaults to <code>10000</code>.</dd>
<dt><code>--json</code></dt>
<dd>Whether to print the results in JSON format. This is good for piping, but not good for human. Specify this flag to enable JSON outputing.</dd>
</dl>

## Notes ##
The query is done in four steps.
1. Parse the query string, build the corresponding abstract syntax tree (AST).
2. Connect to the [API Daemon](/bin/api_daemon/).
3. Translate the AST into a tree-like nested [`Stream`](https://docs.rs/futures/latest/futures/stream/trait.Stream.html).
4. Continuously poll the stream until timeout. Each item the stream yields is a page title, or a warning, or an error.

The output of the query system can be either human readable or machine friendly:
* If a terminal is attached, the output is colored. Each item is printed in a line, warnings are written in yellow and errors in red. At the end of the execution, a summary of the number of yielded pages and warnings and errors is shown.
* If the program is piped to another program like `head`, colors and summaries are suppressed.
* If `--json` flag is set, the output is written in JSON format, with each item being a JSON object. Colors and summaries are suppressed, regardless of whether the program is piped or not.

## Future Work ##
Streams are a current interest of Async Rust Workgroup. It is expected that [`Stream`](https://docs.rs/futures/latest/futures/stream/trait.Stream.html)s or [`AsyncIterator`](https://doc.rust-lang.org/stable/core/async_iter/trait.AsyncIterator.html)s will find their ways into the standard library and become stable.

Currently streams are written with [`async-stream`](https://crates.io/crates/async-stream). They are better written in `Generator`s or [`Coroutine`](https://doc.rust-lang.org/stable/core/ops/trait.Coroutine.html)s. Work is planned to depart from `async-stream` and use builtin `gen` if such language item is generally available.
