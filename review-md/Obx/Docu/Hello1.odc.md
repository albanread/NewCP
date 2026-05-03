**Overview by Example: ObxHello1**

Everything in the BlackBox Component Framework revolves around views. A view is a rectangular part of a document; a document consists of a hierarchy of nested views. What you are now looking at is a text view; below is another text view embedded in it:



  ObxHello1.Do

The embedded text view above contains a slightly more advanced hello world program than *ObxHello0*. It doesn't use module *StdLog* to write into the log window. Instead, it creates a new empty text, to which it connects a text formatter. A text formatter is an object which provides procedures to write variables of all basic Component Pascal types into a text. In the above example, a string and a carriage return are written to the text, which means that they are appended to the existing text. Since a newly created text is empty, *t* now contains exactly what the formatter has written into it.

A text is an object which carries text and text attributes; i.e., a sequence of characters and information about font, color, and vertical offset of each character. However, a text does not know how to draw itself; this is the purpose of a text view. (Yes, what you are currently looking at is the rendering of such a view.) When a text view is created, it receives the text to be displayed as a parameter. Several views can be open on the same text simultaneously. When you edit in one view, the changes are propagated to all other views on the same text.

When you create a text, you perform the steps outlined below:

1) create a text model

2) connect a text formatter to it

3) write the text's contents via the formatter

4) create a text view for the model

5) open the text view in a window

In this example, we have seen how to use the text subsystem in order to create a new text.

