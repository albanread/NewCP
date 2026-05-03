**Overview by Example: ObxParCmd**

This example shows how a command button's context can be accessed. For example, a button in a text may invoke a command, which reads the text immediately following the command. This text can be used as an input parameter to the command.

*ObxParCmd* implements two commands which scan the text behind a command button for a string. In the first command, the string is written to the log:

 "this is a string following a command button"

In this way, tool texts similar to the tools of the original ETH Oberon can be constructed.

In the second example, the string following the button is executed as a command, thus operating in a similar way as a commander (-> DevCommanders):

 "DevDebug.ShowLoadedModules"

[<u>ObxParCmd  sources</u>](../Mod/ParCmd.odc.md)

