**Overview by Example: ObxButtons**

This example demonstrates how new controls can be implemented. A control is a visual object that allows to control some behavior of another part of the program. Examples of controls are buttons, sliders, check boxes, etc. In contrast to full-fledged editors, a control only makes sense in concert with its container and other controls contained therein. With the BlackBox Component Builder, a control is a specialized view. A standard selection of important controls is provided by the BlackBox Component Builder, others can be implemented by third parties. This article shows an example of a simple button control implementation.

Our example control has four properties: its current font, color, link, and label. The link is used by most BlackBox controls to bind them to some program elements. In our case, this program element is a Component Pascal command, i.e. an exported procedure. The label is the string displayed in the button.

Control properties can be inspected and modified with the control property inspector (module *DevInspector*). To try this, select the control, and then use the *Properties...* command in submenu *Object* of menu *Edit* (Windows) / *Part Info* in menu *Edit* (Mac OS).

What do we have to implement to make this control work? In the remainder of this text, we will go through various aspects of the control implementation as it is given in the corresponding listing. We will look at all the procedures which must be implemented for the control to work. As is typical for object- and component-oriented programming, these procedures are meant to be called by the environment, not by yourself. This is called the *Hollywood Principle of Object-Oriented Programming*: Don't call us, we call you.

Typically, a control is embedded in a form. When the form is saved in a file, it gives all embedded controls the opportunity to externalize themselves. When the form is read from a file, the form allocates the controls and lets them internalize themselves. A control programmer needs to implement two procedures for this purpose: *Externalize* and *Internalize*. In the listing you can see that the auxiliary procedures *Views.WriteFont* and *Views.ReadFont* are used. They relieve you from the cumbersome task of writing or reading a font's typeface, size, style, and weight separately.

To support copying operations (e.g., cut and paste), a view must be able to copy its contents to an empty clone of itself. For this purpose, the *CopyFrom* procedure copies the state of an existing control (*source*) to a newly allocated but not yet initialized control.

Our control must be able to redraw its outline and label, which is the purpose of the *Restore* procedure. It draws the label in the control's color and font. The label is horizontally centered. Note that the control doesn't need to draw the background, this is handled by the container. Only the foreground, i.e., the outline and the label are drawn.

Of course, the control should also be able to perform some action when the user clicks in it. Mouse clicks and other interaction events are handled by a view's *HandleCtrlMsg* procedure. In our example, this procedure only reacts upon mouse-down messages (events). Other interactions such as key presses are not handled, in order to make the example as simple as possible.

Basically, the procedure's implementation consist of a mouse tracking loop, which terminates when the user releases the (primary) mouse button. The loop is programmed such that the control's rectangle is inverted as long as the mouse is located over the control, and not inverted otherwise.

If the mouse is released over the control, the inversion is removed and the string in *v.link* passed to the *Dialog.Call* procedure. This procedure basically implements an interpreter for Component Pascal commands, i.e., strings such as "Dialog.Beep" can be executed at run-time.

BlackBox predefines some standard system and control properties. The system properties are the ones that can be changed via the *Characters* menus (*Font*, *Attributes* menus on Mac OS), i.e., font and color. The control properties are the ones defined in module *Controls*, and changeable via the control property inspector. There is a message that BlackBox uses to poll a view's properties (*Properties.PollMsg*), and a message to modify a view's properties (*Properties.SetMsg*). Returning a control's properties is easy: allocate the suitable property descriptors (*Properties.StdProp* and *Controls.Prop*), assign their appropriate fields, i.e., *typeface*, *size*, *style* *weight*, *link*, and *label*, and signal that those are the valid properties that the control knows about. For the standard system properties, this is done in the auxiliary procedure *GetStdProp*. The other corresponding procedure *SetStdProp* is slightly more complicated, because for each property it must be tested whether it needs to be changed or retained. After all, the user may just have changed the control's color without changing the font.

The whole property handling is held together by the view's *HandlePropMsg* procedure, where the *PollMsg* and *SetMsg* messages are recognized. For *SetMsg*, all elements of its property list are traversed and existing system and control properties are picked out. This is done since obviously a control cannot set properties that it doesn't know.

Changes of the control's properties are bracketed by calls to *Views.BeginModification* and *Views.EndModification*. These procedures tell the BlackBox Framework that some operation was performed that cannot be undone. Undo support would not be much more complicated, but still a bit overkill for this example here.

The calls to *Views.Update* signal that the control's visible area should be restored as soon as the current command has terminated.

In *HandlePropMsg*, the control also answers some preferences. Preferences are messages that a container view (e.g., a form) sends to its contained views (e.g., the controls). Their purpose is to customize the container's behavior according to its contents. A container is not obliged to ask embedded views for their preferences, but "socially responsible" containers (e.g., texts and forms) do so. For example, a control which doesn't yet have a defined width or height is asked for its preferred size (*Properties.SizeMsg*). With the *Properties.FocusPref* preference, a view can influence how it is treated when the mouse is clicked in it. Our control indicates that it is a *hot focus*, i.e. it releases its focus immediately after the mouse is released.

Finally, the *New* procedure creates a new control, and the *Deposit* procedure deposits such a new control in a system queue of the BlackBox Framework. For example, the following command sequence can be used to put a new control into this queue:

 "ObxButtons.Deposit; StdCmds.PasteView"

And already we are through with the discussion of the whole control implementation! Even though we haven't addressed some more advanced issues such as keyboard shortcuts for controls, and how a control can be linked to program variables instead of commands, it still is a complete control implementation.

[<u>ObxButtons  sources</u>](../Mod/Buttons.odc.md)

