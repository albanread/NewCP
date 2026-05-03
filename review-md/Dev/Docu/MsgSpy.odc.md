**DevMsgSpy**

A tool that allows to inspect the messages which are sent to a view.

**About the message spy**

*Message Spy* is a tool that logs the messages which are sent to a view through one of the methods *HandleCtrlMsg*, *HandleModelMsg*, *HandlePropMsg*, or *HandleViewMsg*. The message spy displays the type of all messages which are sent to the view and, upon request, also the complete message records.

This tool can help you if your view does not behave as you expect. With the message spy you can learn which messages the frameworks sends to your view and thus which messages you should answer.

**How to inspect a view**

To open the Message Spy, choose *Message Spy... *from menu *Info*. This opens a dialog box similar to the one shown in Figure 1. If you want to inspect a view with the message spy, select it as a singleton and press the *Add View* button. If you want to remove a view from the message spy, select it as a singleton and press the same button, which is now labeled *Remove View*.

Figure 1: The tool window of Message Spy

The tool dialog box of the message spy is divided into two parts. In the upper part, all recognized message types are displayed. This list grows over time. Whenever the message spy meets a message of a new type through any of its inspected views, this type is added to the list of recognized types. Since you might loose the overview over the message types soon, the *Mark New Messages in List *option causes all newly added message types to be selected. In this way you easily learn about messages involved in a particular action. The list of message types can be cleared with the *Clear List* button.

If the option *Show Messages in Log* is selected, then all messages sent to all inspected views whose types are selected in the type selection box are displayed in the lower part of the message spy dialog box. For each message, the type name is displayed followed by a diamond. If this diamond is clicked, then all instance variables of the message record are displayed in a separate window. The log is a regular text which can be scrolled and edited.

**More information**

As message records are static (stack-allocated) variables they have to be copied in order to make them accessible beyond the call of the message handler. The consequence thereof is that only the contents of the message upon message call can be inspected, not how the message is answered by a particular view.

Note that the message spy displays messages of any type sent to an intercepted view, this also includes messages of types which are not exported.

In order to intercept a view, the message spy adds a wrapper around the original view. This wrapper displays the messages and then forwards them to the wrapped view. By replacing a view by its wrapper, the type of the view changes, and thus some tools which depend on a view's type no longer work with inspected views.

Menu command:

    "Message Spy..."    ""    "StdCmds.OpenToolDialog('Dev/Rsrc/MsgSpy', 'Message Spy')"    ""

