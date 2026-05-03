**CommObxStreamsClient**

DEFINITION CommObxStreamsClient;

    PROCEDURE SendTextSelection;

END CommObxStreamsClient.

This module acts as a client for [<u>CommObxStreamsServer</u>](ObxStreamsServer.odc.md). The command sends the current text selection to the server, which opens it in a new TextView. The server is expected to run on the same machine and to be started first. It may, however, run in a BlackBox process separate from the one in which the client runs.

Try (after starting the server and selecting some text):

 CommObxStreamsClient.SendTextSelection

PROCEDURE **SendTextSelection**

Sends a text selection to the server.

[<u>open source</u>](../Mod/ObxStreamsClient.odc.md)

