**Overview by Example: ObxOpen0**

This example asks the user for a file, opens the document contained in the file, and appends a string to the text contained in the document. Finally, the text (in its text view) is opened in a window.



After compilation, you can try out the above example:

  ObxOpen0.Do

With this example, we have seen how the contents of a document's root view can be accessed, and how this view can be opened in a window after its contents have been modified. In contrast to *ObxHello1*, an existing text has been modified; by first setting the formatter to its end, and then appending some new text.

Note that similar to the previous examples, views play a central role.

In contrast to traditional development systems and frameworks, files, windows, and applications play only a minor role in the BlackBox Component Framework, or have completely disappeared as abstractions of their own. On the other hand, the *view* has become a pivotal abstraction. It is an essential property of views that they may be nested, to form hierarchical documents.

These aspects of the BlackBox Component Framework constitute a fundamental shift

  from monolithic applications to software components

  from interoperable programs to integrated components

  from automation islands to open environments

  from application-centered design to document-centered design

