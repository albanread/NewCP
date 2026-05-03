**Overview by Example: ObxAddress0**

One of the hallmarks of modern user interfaces are modeless dialog boxes and data entry forms. They feature text entry fields, push buttons, check boxes, and a variety of other so-called *controls*. The BlackBox Component Builder addresses this issue in a unique way:

ꀢ new controls can be implemented in Component Pascal; they are nothing more than specialized views, there is no artifical distinction between control objects and document objects

ꀢ every view which may contain other views (i.e., a container view) may thus contain controls, there is no artifical distinction between control containers and document containers

ꀢ every general container view can be turned into an input mask when desired

ꀢ some controls can be directly linked to variables of a program

ꀢ linking is done automatically, taking into account type information about the variables in order to guarantee consistency

ꀢ initial form layouts can be generated automatically out of a record declaration

ꀢ forms can be edited interactively and stored as documents, no intermediate code generation is required

ꀢ forms can be used "live" while they are being edited

ꀢ form layouts may be created or manipulated by a Component Pascal program, if this is desired.

Some of these aspects can be demonstrated with the example below:



Compile this module and then execute *New Form...* in the *Controls* menu. A dialog will be opened. Now type "ObxAddress0" into its *Link* field, and then click on the default button. A new window with the form will be opened. This form layout can be edited, e.g., by moving the *update* checkbox somewhat to the right. (Note that you have modified the document by doing that, thus you will be asked whether to save it when you try to close it. Don't save this one.

Now execute the *Open as Aux Dialog* command in the *Controls* menu. As a result, you get a fully functional dialog with the current layout. This mask can be used right away, i.e., when you click into the update check box, the variable *ObxAddress0.adr.update* will be toggled!

If you have several views on the same variable (e.g., both the layout and the mask window), all of them will reflect the changes that the user makes.

A form's controls are linked to a program variable in a similar way as a text view is linked to its text model. Both text views and controls are implementations of the interface type *Views.View*. The container of the above controls is a form view, itself also an implementation of *Views.View*. Form views, just as text views, are examples of container views: container views may contain some intrinsic contents (e.g., text pieces) as well as arbitrary other views. Form views are degenerated in that they have no intrinsic contents; they may only contain other views.

Instead of starting with a record declaration as in the above example, you may prefer to start with the interactive design of a dialog, and only later turn to the programming aspects. This is perfectly possible as well: click on the *Empty* command button to create a new empty form. It will be opened in a new window, and the *Layout* menu will appear. Using the commands in the *Controls* menu, new controls can be inserted into the form. The form can be saved in a file, and its controls may later be connected to program variables using a tool called the control property editor (see the description of the *Edit* menu in the User's Manual).

BlackBox Component Builder only supports modeless forms, whether used as data entry masks or as dialogs. Modal forms would force the user to complete a certain task before doing anything else, e.g., looking up additional information on his task. BlackBox follows the philosophy that the user should be in control, not the computer.

In this example we have seen how a form is used, from the perspective of a programmer as well as from the perspective of a user interface designer. Furthermore we have seen how an initial form layout can be generated automatically; how a form can be viewed (even simultaneously) both as a layout and as a dialog mask; and how a control, like any other view, may live in an arbitrary container view, not just in a form view.

