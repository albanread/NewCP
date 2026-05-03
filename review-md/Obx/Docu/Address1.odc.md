**Overview by Example: ObxAddress1**

This example combines some features of the previous examples: it takes the address record of *ObxAddress1*  and adds behavior to it. Such a record, whose fields are displayed by controls, is called an *interactor*.

The behavior for our example interactor is defined by the global *OpenText* procedure. It creates a new text, into which it writes all the fields of the address record. The fields are written as one line of text, separated by tabulators and terminated by a carriage return. A new text view on this text is then opened in a window.



After the example has been compiled, and after a form has been created for it and turned into a dialog, you can enter something into the fields (note that *customer* only accepts numeric values). Then click on the *OpenText* button. A window will be opened with a contents similar to the following:

Oberon microsystems    Technoparkstrasse 1    Zürich    ZH    8005    Switzerland    1    $TRUE

In this example, we have seen how behavior can be added to interactors, by assigning global procedures to their procedure-typed fields.

