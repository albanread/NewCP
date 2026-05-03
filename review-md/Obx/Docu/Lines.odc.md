**Overview by Example: ObxLines**

This example implements a view which allows to draw lines with the mouse. Beyond line drawing, the only interaction with such a view is through the keyboard: some characters are interpreted as colors, e.g. typing in an "r" sets a view's foreground color (in which the lines are drawn) to red. These two operations certainly don't constitute a graphics editor yet, but can serve as a sketch for a more useful implementation. Full undo/redo of the two operations is supported, however.

The implementation of *ObxLines* is simple: there is a view, a model, and a line data structure. The model represents a view's graph, which consists of a linear list of lines. A line is described by its two end points *(x0, y0)* and *(x1, y1)*. Each view has its own independent foreground color. There is an operation for the entry of a line (a model operation), and an operation for the change of a foreground color (a view operation).

When a line is entered (or the entry is undone), its bounding box is restored. For this purpose, a suitable update message is defined.

There are two interesting aspects of the *ObxLines* implementation:

The contents of a graph, i.e., the linear list of lines, is *immutable*. This means that a line which has been inserted in a graph is never modified again. The list may become longer by adding another line with the  mouse, but the existing line list itself remains unchanged. For example, an undo operation merely changes the graph to show a shorter piece of its line list, redo shows a larger piece of it again.

An immutable data structure has the nice property that it can be freely shared: copying can be implemented merely by letting another pointer point to the same data structure. There is no danger that someone inadvertently changes data belonging to someone else (aliasing problem). This property is used in the example below when copying the model.

The second interesting feature of *ObxLines* is the way *rubberbanding* is implemented. Rubberbanding is the feedback given when drawing a new line, as long as the mouse button is being held down. Procedure *HandleCtrlMsg* shows a typical implementation of such a mechanism: it consists of a polling loop, which repeatedly polls the mouse button's state. When the mouse has moved, the old rubberband line must be erased, and a new one must be drawn.

Erasing the old rubberband line is done using a temporary buffer. Before entering the polling loop, the frame's visible area is saved in the frame's buffer (one buffer per frame is supported). In the polling loop, when the rubberband line must be erased, its bounding box is restored from the buffer. After the polling loop, the last rubberband line is erased the same way, except that the temporary buffer is disposed of simultaneously.

With the *HandlePropMsg* procedure, the view indicates that it may be focused, so it can be edited in-place.

This view has a separate model. Because of this, it implements the "model protocol" of a view, i.e., the methods *CopyFromModelView*, *ThisModel*, and *HandleModelMsg*. *CopyFromModelView* is a model-oriented refinement of *CopyFrom*; it must be implemented instead of *CopyFrom*. *ThisModel* simply returns the view's model. *HandleModelMsg* receives a model update message and translates it into an update of a view's region, which eventually leads to a restoration of this region on the screen.

[<u>ObxLines  sources</u>](../Mod/Lines.odc.md)

 "ObxLines.Deposit; StdCmds.Open"

