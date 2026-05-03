**Overview by Example: ObxCalc**

This example implements a simple pocket calculator view for integer numbers. The calculator can be used either with the keyboard or with the mouse. It is implemented as a stack calculator. The topmost entry of the stack is displayed. With the *^* key (or *ENTER* key on the keyboard) this value is duplicated and pushed onto the stack. The *p*-key performs a pop operation, that is the topmost entry gets replaced by the second one. With the *s*-key the two topmost entries of the stack can be swapped. Expressions must be evaluated using the reverse polish notation. The arithmetic operations replace the two topmost entries by the result. / stands for the quotient and ÷ for the remainder. For example, to add 12 and 25, the keys 12^25+ must be pressed. At the beginning, the stack is filled with zeroes.

The implementation is rather simple. The *ObxCalc* views are views which do not contain a model. Every view keeps its own stack. When a calculator is copied, the state of the copy gets initialized.

The state of a view is not externalized. This implies that any calculator loaded from a file is always in a cleared state.

From a view-programming point of view the following type-bound procedures are of interest:

    Restore    draws the view's contents.



    HandleCtrlMsg    handles all controller messages which are sent to the view, in particular when the mouse button (*Controllers.TrackMsg*) or when a key (*Controllers.EditMsg*) is pressed within the view.



    HandlePropMsg    handles the property messages which are sent to the view by its container. The container asks the view about its (preferred) size (*Properties.SizePref*), whether it is resizable (*Properties.ResizePref*) and whether it wants to become focus or not (*Properties.FocusPref*).



    Externalize    externalizes a version byte



    Internalize    reads the version byte and initializes a new view

With only these five procedures we get a fully working interactive view. The view can be saved to a file (without its internal state) and copied using Drag & Drop or Copy & Paste, it can be printed, and under Windows, it can be exported as ActiveX object into any ActiveX container.

 "ObxCalc.Deposit; StdCmds.Open"

[<u>ObxCalc  sources</u>](../Mod/Calc.odc.md)

