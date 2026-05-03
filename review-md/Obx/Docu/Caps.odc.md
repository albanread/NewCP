**Overview by Example: ObxCaps**

This example shows how the built-in text subsystem of the BlackBox Component Builder can be extended by new commands. To demonstrate this possibility, a command is shown which fetches the selection in the focus view (assuming the focus view is a text view), creates a new empty text, copies the selection into this new text (but with all small letters turned into capital letters), and then replaces the selection by the newly created text. In short, this command turns the selected text stretch into all capital letters.



As an example, select the test string several lines below, and then click on the following commander:

 ObxCaps.Do

teSTstring834 .-st

You'll note that the selection has turned into all caps.

But now comes the surprise: execute *Undo Caps* in the *Edit* menu. As a result, the effect of the uppercase operation is undone! In the BlackBox Component Builder, most operations are undoable; this also holds for the *Delete* and *Insert* text procedures in the above example. Thus, you need to do nothing special in order to render such a command undoable!

However, you may have noticed that there *is* something special in the sample program: there is a *BeginScript* / *EndScript* pair of procedure calls before resp. after the calls to *Delete* / *Insert*. They bundle the sequence of a *Delete* followed by a *Insert* into one single *compound command*, meaning that the user need not undo both of these operations individually, but rather in one single step.

In this example we have seen how an existing displayed text and its selection are accessed, how a buffer text is created, and how the selected text stretch is replaced by the buffer's contents.

