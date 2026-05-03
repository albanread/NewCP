**Oberon by Example: ObxScroll**

By default, a view is always displayed in full size where the preferred size can be specified through the *Properties.SizePref *message. However, sometimes the contents of a view is too large to be shown completely and therefore scrolling of the view's contents within its frame must be supported. An example of views that support scrolling are the text views.

There exists a generic mechanism to implement scrolling by simply changing the scrolled frame's origin. The standard document container uses this feature. Every view which is opened in its own window is contained in a document container, therefore the generic mechanism is available to every root view. The same mechanism could also be offered for embedded views, through a wrapper which scrolls the view's frame on a pixel basis. However, if you want to support a scrolling behavior beyond pixel scrolling, or if the efficient implementation of a view depends on keeping frames small, then explicit handling of scrolling is necessary.

In this example we describe the mechanism ("protocol") a view needs to implement in order to support scrolling. As an example we present the implementation of a 10 by 10 checkers board view (see Figure 1) which can be scrolled within its frame. If this view is scrolled line by line, then the view is displaced field by field. Additionally, this view can also be scrolled if it is embedded in any container that has scrollbars. To scroll an embedded view it must be focus and the modifier key must be hold down when the cursor is over a scrollbar.

Figure 1



If a view wants to support scrolling operations, it must store the current scroll position in its instance variables. In our example, the coordinates of the upper left field (e.g., (6,2)) describe the scroll position. Only the visible fields need to be drawn when the view is restored.

To poll the current scroll state, the framework sends a *Controllers.PollSectionMsg *to the view. According to the information the view returns, the appearance of the scrollbars is determined. In this message, *wholeSize* denotes the width or height of the focus view's contents in arbitrary coordinates, and *partSize* describes the focus view's width or height in the same coordinates (i.e., how large is the view compared to the whole thing). The latter value is used to determine the size of the thumb in the scroll bar (varying thumb sizes are not supported by all window systems). The position of the thumb within the scroll bar is determined according to the value of the field *partPos, *which specifies the view's origin. The value of *partPos* must be greater or equal to zero and smaller or equal to the difference between the whole size and the part size.

In our example, all these values are specified in terms of rows or columns. *partSize* is set to the view's width or height divided by the width or height of one field, and *partPos* is set to the coordinate of the upper left field. For the above example view the part size is four. The value of *wholeSize* depends on the scroll position. If the part size is smaller than the difference between the board size and the part position, then *wholeSize* is set to 10, i.e., to the board size. Otherwise, the part size is greater than the visible fields of the checkers board and this empty space has to be added to the whole size of the view. The latter situation may occur if the view is enlarged, as the scroll position does not change if the view is resized.

When the view is scrolled, a *Controllers.ScrollMsg* is sent to the view. This message specifies whether a horizontal or a vertical scroll operation should be performed, and it also specifies whether the view should scroll by increments or decrements of one line or page, or whether it should scroll to a given absolute position. Here again we must assert that the view is not scrolled outside its frame. If absolute scrolling is performed, then the position to be scrolled to is specified in the same coordinates that the *PollSectionMsg* uses.

All other methods in the source code are straight-forward. The *CopyFrom* method is needed to initialize a copy of the view with the same scroll position, and the *Externalize* and *Internalize* methods allow to make the scroll position persistent. Note, that the scroll position is only persistent for embedded views. For root views, the scroll position is normalized upon externalization, i.e., it is set to (0,0). Whether or not a view should normalize its persistent state is tested with the function *Normalize* which is defined in the context of the view. Embedded views keep their current scroll position when stored, i.e., the scroll position is not normalized.

The *HandlePropMsg* method finally specifies the default size of a newly generated view (depending on its scroll position); it indicates that it wants to become focus (otherwise embedded checkers views could not be scrolled); and it indicates that the size of root views is automatically adapted to the window's size.

The *Deposit* and *DepositAt* commands create and deposit a new checker view. If the scroll position is (x,y), then the default width of the view is 10-x times the cell size and the default height is 10-y times the cell size.

Further improvements of this example are possible. For example, the scroll operations could be implemented as genuine operations that can be undone. Note however, that these operations should only be undoable for views which are not embedded in a root context. Use the *Normalize* method of the context to determine whether the operations need to be undoable or not.

 "ObxScroll.Deposit; StdCmds.Open"

[<u>ObxScroll  sources</u>](../Mod/Scroll.odc.md)

