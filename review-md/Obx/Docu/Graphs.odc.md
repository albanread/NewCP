**Overview by Example: ObxGraphs**

This example is a view which implements simple bar charts. The implementation consists of a view and a model. The view contains the model, and the model contains a linear list of values. For each of these values, a bar is drawn when the view is restored in a frame. The current list of values can be inspected by clicking with the mouse into the view: as a result, an auxiliary window is opened containing a textual representation of the values. This list of numbers can be edited, selected, and then dropped into the graph view again, to change its current list of values. A model's data (the value list) is always replaced as a whole and never modified incrementally. For this reason, not only shallow but also deep copying of a model can be implemented by copying the model reference (the pointer value), rather than by cloning the model and its contents. The same immutability property has been used in example *ObxLines* already.

The most interesting part of this view implementation is the handling of drag & drop messages. In order to let the BlackBox Component Builder provide drop feedback (i.e., the temporary outline denoting a graph view as a possible target for dropping a piece of text) the *Controllers.PollDropMsg* must be handled. In order to actually execute a drop operation (i.e., to let the graph view consume the dropped piece of text), the *Controllers.DropMsg* must be handled. In our example, the dropped view isn't *inserted* into the drop target, but rather *interpreted* as a specification for its new contents.

The handling of the *Properties.FocusPref* preference is also noteworthy. By setting up this preference, a view can inform its container how it would like to be treated concerning focusing. If this message is not handled, a view is never permanently focused, except if it is the root view. This means that upon clicking into the view, the view gets selection handles, but not a focus border. The focus preference allows to modify this behavior.

Graph views are not meant to be targets of menu commands, i.e., it is not necessary to denote them as focusable. Since they should react on mouse clicks into their interior, they shouldn't be selected either. A view which normally is neither focused nor selected, but still should receive mouse clicks in it, is called a *hot focus*. A command button is a more typical example of a hot focus. A view can denote itself as a hot focus by setting the *hotFocus* field of the *Properties.FocusPref* preference to *TRUE*.

Truly editable views should set the *setFocus* field instead of the *hotFocus* field, so the focus is not lost when the user releases the mouse button.

[<u>ObxGraphs  sources</u>](../Mod/Graphs.odc.md)

 "ObxGraphs.Deposit; StdCmds.Open"

After having opened a graph view with the command above, select the list of numbers below



drag the selection over the graph view, and then drop it into the graph view. As a result, the graph view shows a number of grey bars, whose heights correspond to the above values (in millimeters). The bar's widths adapt such that all of them together occupy the whole view's width.

Now click into the graph view to get a list of its current values.

