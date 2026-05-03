**Overview by Example: ObxPatterns**

This view displays a series of concentric rectangles. The view contains no special state; its display is completely determined by its current size alone.

The view implementation is very simple. However, there are three aspects of this program which are noteworthy:

First, although the view has no state, it still extends the *Internalize* and *Externalize* procedures bound to type *View*. This is done only in order to store a version number of the view. It is strongly recommended to store such a version number even in the simplest of views, in order to provide file format extensibility for future versions of the view implementation.

Second, the view contains no special state of its own. However, its display may differ, depending on its current size. A view does not manage its own size, since its size is completely determined by the context in which the view is embedded. A view can determine its current size by asking its context via the context's *GetSize* procedure.

Third, while a view's size is completely controlled by its context, the context still may (but need not!) cooperate when the size is determined. For example, when a view is newly inserted into a container, the container may ask the view for its preferred size; or when it wants to change the embedded view's current size, it may give the view the opportunity to define constraints on its size, e.g., by limiting width or height to some minimal or maximal values. The container can cooperate by sending a size preference (*Properties.SizePrefs*) to the embedded view, containing the proposed sizes (which may be undefined). The view thus gets the opportunity to modify these values to indicate its preferences.

[<u>ObxPatterns  sources</u>](../Mod/Patterns.odc.md)

 "ObxPatterns.Deposit; StdCmds.Open"

Note: in order to change the pattern view's size, first select it using command *Edit->Select Document*, then manipulate one of the resize handles.

