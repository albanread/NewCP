**CommObxStreamsServer**

DEFINITION CommObxStreamsServer;

    PROCEDURE Start;

    PROCEDURE Stop;

END CommObxStreamsServer.

This module provides an example of how to program a server using *CommStreams*. A sample client is provided by the module [<u>CommObxStreamsClient</u>](ObxStreamsClient.odc.md).

The server waits for a connect. Whenever a client requests a new connection a separate TextView is opened by the server and everything received through this connection is displayed in the TextView.

Per machine only one server is permitted to run at a time.

Try - to start / stop the server:

 CommObxStreamsServer.Start

 CommObxStreamsServer.Stop

PROCEDURE **Start**

Starts the server.

PROCEDURE **Stop**

Stops the server.

[<u>open source</u>](../Mod/ObxStreamsServer.odc.md)

