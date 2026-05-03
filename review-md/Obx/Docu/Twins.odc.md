**Overview by Example: ObxTwins**

*ObxTwins* is an example of a container view. Container views come in mainly two flavours: dynamic containers, which may contain an arbitrary and variable number of other views that may be of any type. On the other hand, static containers contain an exactly predetermined number of views, and often the types and layout of these views are predetermined and cannot be changed.

Our example shows the implementation of a static container; for an example of a dynamic container please see the  [<u>form subsystem</u>](../../Form/Docu/Sys-Map.odc.md).

*ObxTwins* implements a view which shows two subviews, a smaller one at the top, and a larger one at the bottom. Both of these views are editable text views; one of them is the current focus. The two views are not treated exactly in the same way: the vertical scroll bar of the twin view always shows the state of the bottom view, even if the top view is focused. There are various ways in which such a twin view can be used. For example, the top view might be used to hold a database query, whose result appears in the bottom view. Or a "chatter" application could be written, with one's own input in the top view, and the partner's messages in the bottom view.

The twin view anchors its embedded views in two *context* objects (extensions of *Models.Context*). Such a context contains a view, its bounding box, and a backpointer to the twin view. A context is the link between a container and a contained view; it allows the contained view to communicate with its container. For this purpose, the contained view has a pointer to its context. Via this pointer, the view can access its environment; e.g., to ask what its current size is, or to ask for a size change or another favour from its container.

A context buffers the bounding box of its view; the twin may change the layout of the contained views by calling procedure *RecalcLayout*, which updates the two context objects in some appropriate way. In a more typical example, the top view would have a constant height - for demonstration purposes, *ObxTwins* shows a more dynamic layout, where the widths remain constant, but the heights are 1/3 for the top view and 2/3 for the bottom view.

The most interesting part of this example is the view's *HandleCtrlMsg* procedure. It handles incoming messages in three different ways. Normally, it just forwards the message to the currently focused subview. Messages which are related to scrolling (*Controllers.PollSectionMsg*, *Controllers.ScrollMsg*, *Controllers.PageMsg*) are always forwarded to the bottom view, independent of the current focus. And finally, cursor messages (*Controllers.CursorMessage*) are always forwarded to the view under the cursor position.

The twin view can be implemented with or without its own model. In this example, an empty dummy model is used. It does not contain state, and it is not externalized/internalized. Its single purpose is to allow more flexible copy behavior. A view without model implements the *CopyFromSimpleView* method, which is always a deep copy. A view with a model implements the *CopyFromModelView* method instead, which may be used for a deep copy, a shallow copy, or the copying with a new model (*Views.CopyWithNewModel*). Note that if you have an application where it is not useful to have several windows open on the same twin view, then you can completely eliminate the twin view's model.

These twins are unorthodox in that their embedded views are not contained in their shared model, as is usual for a normal BlackBox container. Rather, the embedded views are managed (via context objects) directly by the twin views, using separate copies of the embedded views if there are several twin views for the same twin model. The reason for this strange situation is that this allows the embedded views to become visible in different sizes and layouts, depending on the size or other properties of the specific twin view. In this case here, the view sizes are different depending on the twin view size, in particular its height. In effect, the embedded views are shallow-copied and thus behave like they had been opened directly in two different windows of the same document. For example, one text view may have hidden rulers and another one may have visible rulers, and the scrolling positions may be different.

However, note that more orthodox containers would forego this flexibility and use a more standard way of treating the embedded views, by putting them into the twin model.

[<u>ObxTwins  sources</u>](../Mod/Twins.odc.md)

 "ObxTwins.Deposit; StdCmds.Open"

