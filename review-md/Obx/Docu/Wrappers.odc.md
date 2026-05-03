**Overview by Example: ObxWrappers**

This example implements a wrapper. Wrappers are views which contain other views. In particular, a wrapper may have the same size as the view(s) that it wraps. In this way, it can combine its own functionality with that of the wrapped view(s).

For example

ꀢ a debugging wrapper lists the messages received by the wrapped view into the log

ꀢ a background wrapper adds a background color, over which its wrapped view is drawn (which typically has no background color, i.e., which has a transparent background)

ꀢ a layer wrapper contains several layered views, e.g., a graph view overlaid by a caption view

ꀢ a terminal wrapper contains a terminal session and wraps a standard text view displaying the log of the session

ꀢ a bundling wrapper filters out controller messages, such that the wrapped view becomes read-only etc., the sky's the limit! Wrappers demonstrate the power of composition, i.e., how functionality of different objects can be combined in a very simple manner, without having to use complex language mechanisms such as inheritance.

Our example wrapper simply echoes every key typed into the log.

The wrapper implementation is noteworthy in three respects. First, a wrapper reuses the context of the view which it wraps, it has no context of its own. This is only possible because the wrapper is exactly as large as the wrapped view, because a shared context cannot return different sizes in its *GetSize* method.

Second, it uses the wrapped view's model (if it has one!) as its own, this means that its *ThisModel* method returns the wrapped view's model, and that *CopyFromModelView* propagates the new model to the wrapped view.

Third, the wrapper is implemented in a flexible way which allows it to wrap both views with or without models. This flexibility is concentrated in procedure *CopyFromModelView *(which admittedly is a bad name in those special cases where the wrapped views don't have models).

     ObxWrappers.Wrap    *(* select view (singleton) before calling this command *)*

     ObxWrappers.Unwrap    *(* select view (singleton) before calling this command *)*

        *(* <== for example, select this view *)*

and then type in some characters, and see what happens in the log.

[<u>ObxWrappers  sources</u>](../Mod/Wrappers.odc.md)

