# serverless aws websockets

exploration into Rust and [serverless websockets](https://serverless.com/framework/docs/providers/aws/events/websocket/)


## deploy

```sh
$ npm i && npx serverless deploy
```

You can use the `wscat` command line utility to connect and communicate with your
serverless application.

```sh
$ npx wscat -c wss://{YOUR-API-ID}.execute-api.{YOUR-REGION}.amazonaws.com/dev
```

This should open up an interactive repl with which you can communicate with the server

```
connected (press CTRL+C to quit)
> {"action":"send"}
< {"message":"pong"}
```

## how it works

### Traditional servers

A typical websocket server requires an ability to speak a binary protocol over an upgraded
http protocol connection. By its nature that requires th eoperational capability to maintain a
long lived persistant connection with any connected clients. A server with a study security posture should
also provide a means of encrypting information passed across network connections. Secure websocket connects require additional handshake procedures which requires additional binary protocol extensions that you are responsible for.

### Serverless websocket "servers"

API Gateway replaces the need for you to write and operator traditional websocket servers. API Gateway exposing a tls (wss) websocket endpoint and manages persistent connections **for you** freeing you up to focus application specific details. Your application need only implement functions to be invoked to specific lifecycle events called "routes". A few special routes are `$connect` `$disconnect` and `$default` which represent a new client connecting, an existing client disconnecting, and an unmapped request route respectively. You can also route based on a request pattern. By default its expected clients send JSON with an "action" field which gets routed on by value. This example application routes on an action called "send".


Doug Tangren (softprops) 2019