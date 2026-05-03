**Overview by Example: ObxContIter**

This example shows how to iterate over the views embedded in a container, whether it be a text container, a form container, etc. The command searches for the first embedded view whose label is "magic name". At first, the command gets the focus container on which to operate:

    c := Containers.Focus();

This statement obtains the innermost general container's controller in the focus path, even if the container is in mask or browser mode and some control in the container is currently the innermost (non-container) view.

Every general container defines some order for the embedded views. For texts, this is the order of the views in the text, for forms it is the z-ordering, etc. The controller methods *GetFirstView* and *GetNextView* allow to iterate over the embedded views in the defined order:

    c.GetFirstView(Containers.any, v);

    c.GetNextView(Containers.any, v);

The first parameters denote whether all embedded views should be traversed (*Containers.any*) or only the selected views (*Containers.selection*).

The loop that searches for the specific control has the typical form of a search loop:

    get first element;

    WHILE (element # NIL) & ~(element is the right one) DO

        get next element

    END;

In our example, the condition in the WHILE loop is the following:

    (v # NIL) & ~((v IS Controls.Control) & (v(Controls.Control).label = "magic name"))

The right-side term contains an expression that denotes "element is the right one", in this case:

    (v IS Controls.Control) & (v(Controls.Control).label = "magic name")

This means that the element is a control, and its label is "magic name". The not-operator "~" inverts this condition, so that the loop continues as long as the right element has not yet been found.

Note that you shouldn't modify the selection during iteration over the selection. Also, you shouldn't insert into or delete from the container model during iteration.

[<u>ObxContIter  sources</u>](../Mod/ContIter.odc.md)

