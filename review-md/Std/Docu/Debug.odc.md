**StdDebug**

DEFINITION StdDebug;

END StdDebug.

For a distribution version of an BlackBox application or component, use this limited version of the BlackBox debugger. It is not permitted to distribute the full debugger (*DevDebug*). This limited version doesn't allow to follow pointers and similar interactions, it only creates a passive display of the error state. Since this is a text document that can be saved, it enables the user to send error information to the developer (e.g., via *StdCoder*).

*StdDebug* is installed during startup of BlackBox, if the full *DevDebug* could  not be found.

