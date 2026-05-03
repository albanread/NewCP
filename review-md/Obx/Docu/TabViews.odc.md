**Overview by Example: ObxTabViews**

The property inspector for *StdTabViews* supplies a graphical user interface for creating and editing *StdTabViews*. This user interface is sufficient for most applications. However, if tabs need to be added and removed dynamically during runtime, the programming interface of *StdTabViews* needs to be used.

This example creates a *StdTabViews.View* with three tabs to start with. Then new tabs can be added or the selected tab can be deleted in runtime. Any kind of views can be added to a *StdTabViews.View*. In this example one *ObxCube*-view is added and the same *ObxCalc*-view is added several times. This shows another charactaristic of *StdTabViews*; when a view is added it is copied. Eventhough the same view is used to create several tabs, they all have their own view internally. You can easily try this by typing a number into a calculator view, then changing tab to another calculator view and type in another number. When you switch back to the first calculator it still displays the first number.

The example also installs a simple notifier which simply displays the notify message in the log.

Here you can try the example:

 "ObxTabViews.Deposit; StdCmds.Open"

To add a new calculator view, use this command:

 ObxTabViews.AddNewCalc

To delete the selected tab, use this command:

 ObxTabViews.DeleteTab

[<u>ObxTabViews  sources</u>](../Mod/TabViews.odc.md)

