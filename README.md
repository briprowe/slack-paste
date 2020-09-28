# slack-paste

Paste stdin to slack.

## Initial Setup

You can use `cargo` to install `slack-paste`:

```sh
cargo install slack-paste
```

After `slack-paste` is installed you have to configure it to work with
your slack workspace. To do so, either use an oauth token for an
existing bot in your workspace, or create a new bot.

Navigate to https://api.slack.com/apps/ and create a new app.

Once you have an oauth access token with the `chat:write` scope, you
can configure `slack-paste` to use it with the following command.

```sh
slack-paste init
```

## Usage

```sh
cat my-file.rs | slack-paste paste @coworker
```
