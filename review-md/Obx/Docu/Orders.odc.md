**Overview by Example: ObxOrders**

*ObxOrders* is a more realistic example of a program which uses forms for data entry and data manipulation. It lets the user enter new orders, browse through existing orders, and save all orders into a file, or load them from a file. A dialog always shows the current order. For this order, an invoice text can be generated, ready to be printed.

    The order data is stored in a very simple main memory database: each order is represented as a record, and the orders are connected in a doubly-linked ring with a dummy header element. One element of a ring thus contains a *next* and a *prev* pointer, as well as the data of one order. In this case, the data is directly represented as a variable of the interactor type which is used for data entry. Note that this approach is only possible in very simple cases; normally an interactor would only represent a subset of a tuple stored in the database, and the database representation would be independent of any specific interactor.

    The other aspect of our example program which is simpler than in typical applications is that the database is global: the doubly-linked ring is anchored in a global variable, so there is at most one order database open at any given point in time.

This example has four noteworthy aspects: The procedure *NewRuler* shows how a new text ruler with specific properties can be created, and in procedure *Invoice* it can be seen how the text font style can be changed to bold and back to normal when creating a text.

    Thirdly, the dialog uses *guard commands* to disable and enable controls in the dialog: Depending on whether there currently is an open database, the data entry, invoice generation, etc. controls are enabled, otherwise disabled. Depending on whether the current order is the last (first) one, the *Next (First)* button is disabled, otherwise enabled.

    Fourth, the formatter procedure *WriteView* is used twice to insert a view into the text: first a ruler view is written, and a bit later a standard clock view is written to the text.

[<u>ObxOrders  sources</u>](../Mod/Orders.odc.md)

[<u>Main dialog</u>](../Rsrc/Orders.odc.md)

[<u>"Delete" dialog</u>](../Rsrc/Orders1.odc.md)

 "StdCmds.OpenAuxDialog('Obx/Rsrc/Orders', 'Order Processing')"

In Obx/Samples/Odata there is some sample data.

