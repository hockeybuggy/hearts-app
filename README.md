# hearts-app

A card game app using websockets.


## Deploy

This app is built using the serverless framework deploying to AWS. It is based
on [this repo](https://github.com/softprops/serverless-aws-rust-websockets).

This app currently doesn't have a "local mode" and it deployed to one
environment "dev". To deploy the app:

```sh
npm run deploy:dev
```

## Using the app

You can use the `wscat` command line utility to connect and communicate with
your serverless application.

```sh
npm run wscat:dev
```

This should open up an interactive repl with which you can communicate with the
server

You can send messages to the server with a json payload containing an "action"
field of "send" with an optional text "message" field

```
connected (press CTRL+C to quit)
> {"action":"send"}
< {"message":"🏓 pong"}
> {"action":"send", "message":"psst"}
< {"message":"psst"}
```
