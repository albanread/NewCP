**Part II: Library**

Part I of the BlackBox tutorial gives an introduction to the design patterns that are used throughout BlackBox. To know these patterns makes it easier to understand and remember the more detailed design decisions in the various BlackBox modules.

Part II of the BlackBox tutorial demonstrates how the most important library components can be used: control, form, and text components.

Part III of the BlackBox tutorial demonstrates how new views can be developed, by giving a series of examples that gradually become more sophisticated.

**4 Forms**

In the first chapter of this part, we concentrate on the forms-based composition of controls. "Forms-based" means that there exists a graphical layout editor, sometimes called a visual designer or screen painter, that allows to insert, move, resize, and delete controls. Controls can be made active by linking them to code pieces. For example, it can be defined what action should happen when a command button is clicked.

Controls can have different visible states, e.g., they can be enabled or disabled. This is a way to inform a user about which actions currently make sense. Setting up control states in a sensible way can make a large difference in the user-friendliness of an application. Unfortunately, these user interface features often require more programming effort than the actual application logic itself does. However, there are only a few concepts necessary to understand in order to build such user interfaces. These concepts will be explained by means of simple examples.

**4.1 Preliminaries**

We want to start as quickly as possible with a concrete example, but a few preliminary remarks are still in order. The reader is expected to have some basic knowledge of programming, preferably in some dialect of Pascal. Furthermore, he of she is expected to know how to use the platform (Windows or Macintosh), and the general user interface guidelines for the platform.

The BlackBox Component Builder is used as the tool to expose the various characteristics of component software in an exemplary way. The contents of the book applies both to the Windows and the Macintosh version of the BlackBox Component Builder; except for screendumps, which mostly use Windows. In order to minimize platform-specific remarks in the text, a few notational conventions are followed:

ꀢ Mac OS folders are called directories

ꀢ Path names contain "/" as directory separators, as in Unix and the World-Wide Web

ꀢ File and directory names may contain both capital and small letters

ꀢ Document file names are given without the ".odc" suffix used in Windows. Thus the file name Text/Rsrc/Find under Windows corresponds to Text\Rsrc\Find.odc on Windows, and to Text:Rsrc:Find under Mac OS.

ꀢ Modifier key: on Windows this is the Ctrl key, on Mac OS it is the Option key

ꀢ Menu commands: *M->I* is a shorthand notation for menu item I in menu M, e.g., *File->New*

When working in the BlackBox environment for the first time, the following may be helpful to remember: almost all modifications to BlackBox documents are undoable, making it quite safe to try out a feature. In general, multi-level undo is available; i.e., not only one, but several commands can be undone; as many as memory permits.

This book is not a replacement for the BlackBox user manual. It minimizes the use of specific BlackBox Component Builder tool features, and therefore the need for tool-specific descriptions. The text is intended to concentrate on programming, rather than on the tool. Where unavoidable, tool-specific explanations are given where they are first needed.

Most examples are also available on-line in the Obx/Mod directory of BlackBox. They can be opened and compiled immediately. At the end of every section, references to further Obx on-line examples for the same topic are given.

For readers who are not yet fluent in Component Pascal, Appendix B describes the differences beween Pascal and Component Pascal.

The Help screen of the BlackBox Component Builder gives direct or indirect access to the complete and extensive on-line documentation, e.g., to the user manual, to all the Obx ("**O**verview **b**y e**x**ample") examples, and so on. On the Web, additional resources may be found at http://www.oberon.ch.

**4.2 Phone book example**

Throughout this part of the book, we will meet variations of and additions to a specific example. The example is an exceedingly simple address database. The idea is not to present a full-fledged application with all possible bells and whistles, but rather a minimal example which doesn't hide the concepts to be explained behind large amounts of code. Nevertheless, the idea of how bells and whistles can be added, component by component, should become obvious over time.

Our phone book database contains the following fixed entries:

    Daffy Duck    310-555-1212

    Wile E. Coyote    408-555-1212

    Scrooge McDuck    206-555-1212

    Huey Lewis    415-555-1212

    Thomas Dewey    617-555-1212

Table 4-1. Entries in phone book database

The database can be searched for a phone number, given a name. Alternatively, the database can be searched for a name, given a phone number; or its contents can be accessed by index. We have met a possible implementation of this database in section 3.2.

Our goal is to create a user interface for the database. The user interface is a dialog box that contains two edit fields; one for the name, and the other for the phone number. For each text field, there is a caption that indicates its purpose. Furthermore, the dialog box contains a check box which allows to specify whether the name or the phone number should be looked up, and finally there is a command button which invokes the lookup command. The following screendump shows how the dialog box will look like eventually:

Figure 4-2. Phone book database mask

To construct this mask, we start by creating a new empty form for the dialog box, using command *Controls->New Form*. This results in the following dialog box:

Figure 4-3. New Form dialog box

Clicking on the *Empty* command button opens an empty form:

Figure 4-4. Empty form

Using the commands of menu *Controls* , we insert the various controls we need, i.e., two captions, two edit fields, a check box, and a command button. The controls we've inserted still have generic labels such as "untitled" or "Caption". To change these visual properties, a control property inspector is used. It is opened by selecting a control and then issuing *Edit->Object Properties...* (Windows) or *Edit->Part Info* (Mac OS), respectively. Edit the "label" field in order to change the selected control's label, and click on the default button to make the change permanent.

Figure 4-5. Control property editor

Change the "label" field of each control so that you end up with a layout similar to the one of Figure 4-2. Listed in a tabular way, the labels are the following ones (from left to right, top to bottom):

*    Control type    Label*

    Caption    Name

    Text Field

    Caption    Number

    Text Field

    Check Box    Lookup by Name

    Command Button    Lookup

Table 4-6. List of controls in phone book dialog box

The controls can be rearranged by using the mouse or by using the layout commands in menu *Layout*. After having edited the layout, make sure to call *Layout->Sort Views*; this command sorts the controls in such a way that when being pressed, the tabulator key moves the cursor between them in the order you would expect, i.e., from left to right and from top to bottom.

Figure 4-7. Completed layout of phone book dialog box

When you are happy with the layout, you can save the dialog box just like any other document, by using *File->Save*. As a convention, the dialog box layouts of all examples are saved in directory Obx/Rsrc. In this case, we save the new dialog box as Obx/Rsrc/PhoneUI.

The directory name *Rsrc* stands for "Resources". Resources are documents that are necessary for a program to work. In particular, they are dialog box layouts and string resources. String resources allow to move strings out of a program's source code into a separately editable document.

Resources can be edited without recompiling anything. For example, you could change all labels in the above dialog box from English to German, without having access to any source code or development tool.

**4.3 Interactors**

Creating a dialog box layout is fine, but there also must be a way to add behavior to the dialog box. In our example, user interactions with the dialog box should lead to lookup operations in the phone book database. To achieve this, we need an actual implementation of the database. We mentioned earlier that a suitable implementation already exists. This is not surprising, since in this part of the book we talk about component object assembly, which means taking existing components and suitably integrating persistent objects that they implement.

In our example, the phone book database is a Component Pascal module called *ObxPhoneDB.* We have already met this module in Chapter 3. Now we want to write our first new module, whose purpose is to build a bridge between the database module *ObxPhoneDB* and the dialog box we've built earlier. The new module is called *ObxPhoneUI*, where "UI" stands for "user interface". It is a typical script module whose only purpose is to add behavior to a compound document, such that it can be used as a front-end for the application logic, which in our case is simply the phone book database.

The new module uses, i.e., imports, two existing modules. On the one hand, it is the *ObxPhoneDB* module. On the other hand, module *Dialog*:

Figure 4-8. Import relation between ObxPhoneUI and ObxPhoneDB

*Dialog* is part of a fundamental framework coming with the BlackBox Component Builder. The module provides various services that support user interaction. We'll meet the most important ones in this chapter.

MODULE ObxPhoneUI;

    IMPORT Dialog, ObxPhoneDB;

    VAR

        **phone***: RECORD

            name*, number*: ObxPhoneDB.String;

            lookupByName*: BOOLEAN

        END;

    PROCEDURE **Lookup***;

    BEGIN

        IF phone.lookupByName THEN

            ObxPhoneDB.LookupByName(phone.name, phone.number);

            IF phone.number = "" THEN phone.number := "not found" END

        ELSE

            ObxPhoneDB.LookupByNumber(phone.number, phone.name);

            IF phone.name = "" THEN phone.name := "not found" END

        END;

        Dialog.Update(phone)

    END Lookup;

END ObxPhoneUI.

Listing 4-9. First version of ObxPhoneUI

*ObxPhoneUI* exports a global record variable *phone*, which contains two string fields and a Boolean field. Depending on the current value of the Boolean field, the *Lookup* procedure either takes *phone.name* to look up the corresponding number, or *phone.number* to look up the corresponding name. If the lookup fails, i.e., returns the empty string, the result is turned into "not found". Either result is put into *phone*, i.e., *phone* carries both input and output parameters for the database lookup.

Since the user could change the contents of *phone* by interactively manipulating a control, e.g., by typing into a text entry field, a global variable like *phone* is called an *interactor*. Controls display the contents of their interactor fields and possibly let them be modified interactively. To do this, every control must first be linked to its corresponding interactor field. This is done with the control property editor that we have seen earlier. Its "Link" field should contain the field name, e.g., *ObxPhoneDB.phone.name* or a procedure name such as *ObxPhoneDB.Lookup*. Use the control property inspector to set up the link fields according to Table 4-10:

*    Control type    Label    Link*

    Caption    Name

    Text Field        ObxPhoneUI.phone.name

    Caption    Number

    Text Field        ObxPhoneUI.phone.number

    Check Box    Lookup by Name    ObxPhoneUI.phone.lookupByName

    Command Button    Lookup    ObxPhoneUI.Lookup

Table 4-10. Links of phone book dialog box

Note that the previously disabled controls now have become enabled. What does this mean? When a control is linked, which normally happens when it is being read from a file, or in our case when the link is changed by the inspector, then the module to which the control should be linked must be loaded. If it is already loaded, nothing needs to be done. If it isn't loaded yet (remember that you can check with *Info->Loaded Modules*), loading is done now. If loading fails, e.g., because the module's code file doesn't yet exist, then control linking fails and the control remains disabled. Linking also fails if the control and field types don't match, e.g., if a check box is linked to a string field.

To achieve this level of functionality (and safety against incorrect use), BlackBox provides several advanced "metaprogramming" services, in particular dynamic module loading on demand and typesafe lookup of variables. The latter requires extensive run-time type information (RTTI) that is relatively uncommon in fully compiled languages.

The links of all controls are reevaluated whenever a module has been unloaded. This ensures that controls are never linked to unloaded modules.

It was one of the design goals for BlackBox to separate user interface details from program logic. For this reason, module *ObxPhoneUI* doesn't know about controls and forms and the like. Instead, the controls of our form have links, which tell them the interactor fields with which they should interact. For example, the command button's "ObxPhoneUI.Lookup" link tells it to activate the *ObxPhoneUI.Lookup* procedure when the button is pressed. This procedure in turn doesn't know about the command button (there even may be several of them), the only thing it does to acknowledge the possible existence of controls is to call

    Dialog.Update(phone)

at the end of the command procedure. *Dialog.Update* causes an update of all controls that need updating. For example, if *Lookup* has assigned "not found" to *phone.number*, the corresponding text field(s) or similar controls need to be redrawn accordingly.

As parameter of *Dialog.Update*, an interactor must be passed. Calling *Dialog.Update* is necessary after one or several fields of this interactor have been modified by a program. If several fields have been modified, *Dialog.Update* should only be called once, for efficiency reasons. Note that a control calls *Dialog.Update* itself when the user has modified an interactor field; you only need to call it after your own code has modified the interactor.

This strong separation of user interface from program logic is uncommon. Its advantage is simplicity: as soon as you know how to define procedures and how to declare record types and global variables, you can already construct graphical user interfaces for modules. This is possible even for someone who is just beginning to learn programming. Another advantage is that you have to write no code for simple user interfaces. User interface construction happens in the forms editor (e.g., setting the position and size of a control) and with the inspector (e.g., setting the alignment of text in a text field). This makes it easier to adapt an application to different user interface requirements, without touching the application logic itself. Only if you want to exercise more control over the user interface, e.g., disabling controls or reacting on special events such as the user's typing, then you need to write small amounts of code, which can be very cleanly separated from the application logic itself. The necessary concepts, so-called guards and notifiers, will be discussed in the next two sections. If you need still more control, then you can access controls individually, as described in section 4.9.

Currently, a disavantage of the BlackBox Component Builder's approach is that all controls have to be linked to global interactor variables. If there are several controls for the same interactor field, all of them display the same value. The controls cannot have independent state of their own.

Note an interesting feature of BlackBox: if you have written a module like *ObxPhoneUI* in Listing 4-9, you can automatically generate a form with suitable controls in a default layout. This is done by clicking "Create" in the "New Form" dialog box instead of "Empty". This feature is useful to create temporary testing and debugging user interfaces during development, where it isn't useful to spend time with manual form construction.

We now have a dialog box layout with controls linked to module *ObxPhoneUI*, and this module imports the database engine *ObxPhoneDB*. What is still missing is a way to *use* the dialog box, rather than to merely edit its layout. During editing, it can be useful to immediately try out the dialog box, even before its layout is perfect. To try this out, make sure that the layout window is on top and then execute *Controls->Open As Aux Dialog*. A new window is opened which contains the same dialog box, but in a way that its *controls* can be edited, rather than its *layout*. The window acts as a data entry mask. Now you can type, for example, "Huey Lewis" into the name string, click on the "Lookup by Name" check box, and then click on the "Lookup" button. You'll see that the appropriate phone number appears in the "Number" field.

Figure 4-11. Layout view (left) and mask view (right) displaying the same form model

Note that the same name and number also appeared in the layout window. Even better, if you change the layout in the layout window, e.g., by moving the check box somewhat, you'll note that the layout change is immediately reflected in the other window. This is a result of the so-called Model-View-Controller implementation of BlackBox. In Part II of the book, we have discussed this design pattern in more detail. Here it is sufficient to note that several views can share the same data, e.g., a layout view and a mask view can display the same form; and that both layout and mask views are basically the same kind of view albeit in different modes. You can switch between these modes by applying the *Dev->Layout Mode* or *Dev->Mask Mode* commands.

The two modes differ in the ways they treat selection and focus. In layout mode, you can select the embedded views and edit the selection, but you cannot focus the embedded views. In mask mode, you can focus the embedded views, but you cannot select them (only their contents) and thus cannot edit their layout, i.e., their sizes, positions, etc. In other words: layout mode prevents focusing, while mask mode prevents selection.

Opening a second view in mask mode for our form layout is convenient during layout editing, but you wouldn't want any layout view open when your program should actually be *used*. In this case, you want to open the dialog box in mask mode by invoking a suitable menu command.

A new menu command can be introduced by editing a menu configuration text. You can open this text (which resides in System/Rsrc/Menus) by calling *Info->Menus*. Append the following text to its end:

    MENU "Priv"

        "Open..."    ""    "StdCmds.OpenAuxDialog('Obx/Rsrc/PhoneUI', 'Phonebook')"    ""

    END

Having done this, execute *Info->Update Menus*. You'll notice that the new menu "Priv" has appeared. Execute its menu item "Open...". As a result, the command

    StdCmds.OpenAuxDialog('Obx/Rsrc/PhoneUI', 'Phonebook')

will be executed. It opens the Obx/Rsrc/PhoneUI layout document, turns it into mask mode, and opens it in a window with title "Phonebook". If you want the modification of your menu text to become permanent, save the "Menus" text before closing it.

Note that the form has not been saved in mask mode (this would be inconvenient for later editing), it is only temporarily turned into mask mode by the *StdCmds.OpenAuxDialog* command.

**4.4 Guards**

In terms of genuine functionality, we have seen everything that is important about the standard set of controls. We will look at the palette of standard controls in more detail later. However, we first need to discuss important aspects of standard controls which, strictly speaking, do not increase the functionality of an application, but rather its useability, i.e., its user-friendliness.

In section 1.2 we have already discussed what user-friendliness means. For example, it means avoiding modes wherever possible. For this reason, BlackBox doesn't support modal dialog boxes.

Modes are unavoidable if a user action sometimes makes sense, and sometimes doesn't. For example, if the clipboard is empty, its contents cannot be pasted into a text. If it is not empty and contains text, pasting is possible. This cannot be helped, and is harmless if the current state is clearly visible or can easily be inquired by the user. For example, if the clipboard is empty, the *Paste* menu command can be visibly marked as disabled. The visual distinction gives the user early feedback that this command is currently not meaningful. This is usually much better than to let the user try out a command and then give an error message afterwards.

The following example shows a dialog box with two buttons. The *Empty* button is enabled, while the *Create* button is disabled. The *Create* button only becomes enabled if something has been typed into the dialog box's text field:

Figure 4-12. Enabled and disabled buttons

In summary, a good user interface always lets the user perform every meaningful action, and gives visual cues about actions that are not meaningful.

For the BlackBox, we thus need a way to provide feedback about the current state of the system, especially about which commands are currently possible and which aren't.

For this purpose, it must be possible to enable and disable controls and menu items. For example, looking up a phone number is only possible if some name has been entered, i.e., if the name field is not empty. To determine whether a command procedure, in our case the *Lookup* procedure, may be called, i.e., whether a corresponding control or menu item may be enabled, a suitable *guard* must be provided. A guard is a procedure called by the framework, whenever it might be necessary to change the state of a control or menu item. The guard inspects some global state of the system, uses this state to determine whether the guarded command currently makes sense, and then sets up an output parameter accordingly. A guard has the following form:

    PROCEDURE XyzGuard* (VAR par: Dialog.Par);

    BEGIN

        par.disabled := *...some Boolean expression...*

    END XyzGuard;

A guard has the following type:

    GuardProc = PROCEDURE (VAR par: Dialog.Par);

To guard procedure *Lookup* in our example, we extend module *ObxPhoneUI* in the following way:

MODULE ObxPhoneUI;

    IMPORT Dialog, ObxPhoneDB;

    VAR

        **phone***: RECORD

            name*, number*: ObxPhoneDB.String;

            lookupByName*: BOOLEAN

        END;

    PROCEDURE **Lookup***;

    BEGIN

        IF phone.lookupByName THEN

            ObxPhoneDB.LookupByName(phone.name, phone.number);

            IF phone.number = "" THEN phone.number := "not found" END

        ELSE

            ObxPhoneDB.LookupByNumber(phone.number, phone.name);

            IF phone.name = "" THEN phone.name := "not found" END

        END;

        Dialog.Update(phone)

    END Lookup;

    PROCEDURE **LookupGuard*** (VAR par: Dialog.Par);

    BEGIN    *(* disable if input string is empty *)*

        par.disabled := phone.lookupByName & (phone.name = "") OR

                                    ~phone.lookupByName & (phone.number = "")

    END LookupGuard;

END ObxPhoneUI.

Listing 4-13. ObxPhoneUI with LookupGuard

What happens if we compile the above module? Its symbol file on disk is replaced by a new version, because the module interface has been changed from the previous version. Because the change is merely an addition of a global procedure (i.e., the new version is compatible with the old version) possible client modules importing *ObxPhoneUI* are not invalidated and need not be recompiled.

Compilation also produced a new code file on disk. However, the old version of *ObxPhoneUI* is still loaded in memory! In other words: once loaded, a module remains loaded ("terminate-and-stay-resident"). This is not a problem, since modules are extremely light-weight and consume little memory. However, a programmer of course must be able to unload modules without leaving the BlackBox Component Builder entirely, in order to try out a new version of a module. For this purpose, the command *Dev->Unload* is provided which unloads the module whose source code is currently focused.

Note that compilation does not automatically unload a module, since this is often undesirable. In particular, as soon as you work on several related modules concurrently, unloading one of them before the others are correctly updated would render this whole set of modules inconsistent.

For those cases where immediate unloading after compilation *does* make sense, like in the simple examples that we are currently discussing, the command *Dev->Compile And Unload* is provided.

You may use this command to try out our new version of *ObxPhoneUI*. Notice how the *Lookup* button is disabled when the *Name* field is empty (if *Lookup by Name* is chosen) or when the *Number* field is empty (if *Lookup by Number* is chosen). Typing something into the field makes *Lookup* enabled again; deleting all characters in the field disables it again. At the end of this section we will explain more precisely when guards are evaluated; for now it is sufficient to know that they are evaluated after every character typed into a text field control.

Guards are mostly used to enable and disable user interface elements such as controls or menu items. However, they sometimes play a more general role as well. For example, controls may not only be disabled, but they also may be made read-only or undefined.

*Read-only* means that a control currently cannot be modified interactively. For example, the following guards set up the output parameter's *readOnly* field. The first guard sets read-only if lookup is not by name, i.e., by number. Obviously, in this case a number is input and a name is output. Pure outputs should be read-only. Thus the first guard can be used as guard for the *Name* field. The second guard can be used for the *Number* field, since it sets read-only if lookup returns a number.

    PROCEDURE **NameGuard*** (VAR par: Dialog.Par);

    BEGIN    *(* make read-only if lookup is by number *)*

        par.readOnly := ~phone.lookupByName

    END NameGuard;

    PROCEDURE **NumberGuard*** (VAR par: Dialog.Par);

    BEGIN    *(* make read-only if lookup is by name *)*

        par.readOnly := phone.lookupByName

    END NumberGuard;

Listing 4-14. Read-only guards for ObxPhoneUI

Interactor fields which are exported read-only, i.e., with the "-" export mark instead of the "*" export mark, are always in read-only state regardless of what a guard specifies (otherwise this would violate module safety, i.e., the invariant that a read-only exported item can only be modified within its defining module).

The *undefined* state of a control means that the control currently has no meaning at all. This happens if a control displays the state of a heterogeneous selection. For example, a check box may indicate whether the text selection is all caps or all small letters. If part of the selection is capital letters and the rest small letters, then the control has no defined value. However, by clicking on the check box, the selection is made all caps and then has a defined state again. The undefined state can be set by a guard with the statement

    par.undef := TRUE

The undefined state can be regarded as "write-only", i.e., the control's state cannot be read by the user because it currently has no defined value, but it can be modified and thus set to a defined value.

Of the fields *disabled*, *readOnly* and *undef*, at most one may be set to *TRUE* by a guard. This leads to four possible temporary states of the control: it is in none or in exactly one of the three special states. When a guard is called by the framework, all three Boolean fields are preset to *FALSE*. This is a general rule in BlackBox: Boolean values default to *FALSE*.

In Table 4-15, suitable guards for the layout of Figure 4-2 are listed:

*    Control type    Link    Guard*

    Caption

    Text Field    ObxPhoneUI.phone.name    ObxPhoneUI.NameGuard

    Caption

    Text Field    ObxPhoneUI.phone.number    ObxPhoneUI.NumberGuard

    Check Box    ObxPhoneUI.phone.lookupByName

    Command Button    ObxPhoneUI.Lookup    ObxPhoneUI.LookupGuard

Table 4-15. Guards in phone book dialog box

A guard applies to a procedure, to an interactor field, or to several procedures or interactor fields. If it applies to mainly one procedure or field, the guard's name is constructed by appending "Guard" to the (capitalized) name of the procedure/field. For example, the guard for procedure *Lookup* is called *LookupGuard*, the guard for field *phone.name* is called *NameGuard*, etc. This is a simple naming convention which makes it easier to recognize the relation between guard and guarded item. The naming convention is "soft", i.e., it is not enforced by the framework; in fact the framework doesn't know about it at all.

It is not a convention, but a necessity, to export guards. Guards are accessed by the so-called metaprogramming mechanism of BlackBox which, for safety reasons, only operates on exported items. This is consistent with the treatment of a module as a black-box, of which only the set of exported items, i.e., its interface, is accessible from the outside. If a guard isn't exported, a control cannot call it. This is similar to the fields of an interactor, which also must be exported if an interactor should be able to link to it.

If we look at the declaration of type *Dialog.Par* (use *Info->Interface*!), we see that two fields have not yet been discussed: *label* and *checked*:

    Par = RECORD

        disabled, checked, undef, readOnly: BOOLEAN;

        label: Dialog.String

    END

Field *label* allows to change the label of a control. For example, instead of using the label "Toggle" for a button it may be more telling to use the labels "Switch On" and "Switch Off" depending on the current state of the system. This can be done by a procedure like this:

    PROCEDURE **ToggleGuard*** (VAR par: Dialog.Par);

    BEGIN

        IF someInteractor.isOn THEN

            par.label := "Switch Off"

        ELSE

            par.label := "Switch On"

        END

    END ToggleGuard;

Listing 4-16. Label guard example

Note that the guard overrides whatever label was set up in the control property inspector. This is true for all controls (and for menu items as well, see below).

It is strongly recommended not to place string literals in the source code like in the above example, because this would force a recompilation of the code if the language were changed, e.g., from English to German (see also section 1.4). In BlackBox, user interface strings such as labels or messages are generally packed into separate parameter files, so-called string resources. For each subsystem there can be one string resource file, e.g., Text/Rsrc/Strings or Form/Rsrc/Strings. A string resource file is a BlackBox text document starting with the keyword *STRINGS* and followed by an arbitrary number of *<key, string>* pairs. The key is a string, separated by a tab character from the actual string into which it will be mapped. Every *<key, string>* pair must be terminated by a carriage return. The pairs need not be arranged in any particular order, although it is helpful to sort them alphabetically by key, because this makes it easier to find a particular key when editing the string resources.

For example, System/Rsrc/Strings starts the following way:

STRINGS

About    About BlackBox

AlienAttributes    alien attributes

AlienCause    alien cause

AlienComponent    alien component

AlienControllerWarning    alien controller (warning)

...

Table 4-17. String resources of System/Rsrc/Strings

To use string resources in our guard example, a special syntax must be used to indicate that the string is actually a key that first must be mapped using the appropriate subsystem's string resources. Assuming that in the *Obx* subsystem's string resources there exist "On" and "Off" keys, the following code emerges:

    PROCEDURE **ToggleGuard*** (VAR par: Dialog.Par);

    BEGIN

        IF someInteractor.isOn THEN

            par.label := "#Obx:Off"

        ELSE

            par.label := "#Obx:On"

        END

    END ToggleGuard;

Listing 4-18. Label guard example with string mapping

The leading "#" indicates that a string mapping is desired. It is followed by the subsystem name, in this case "Obx". Then comes a colon, followed by the key to be mapped. A command button with this guard would either display the label "Switch Off" or "Switch On" in an English version of BlackBox, "Ausschalten" or "Einschalten" in a German version, and so on.

If there is no suitable string resource, the key is mapped to itself. For example, if there is no "Off" key in Obx/Rsrc/Strings, then "#Obx:Off" will be mapped to "Off".

The remaining field of *Dialog.Par* that we have not yet discussed is called *checked*. Actually, so far it has never been used for controls in BlackBox. It is used for menu items. Menu items are similar to controls: they can invoke actions, they may be enabled or disabled, and they have labels. For this reason it makes sense to use the same guard mechanism for them also. A menu guard is specified in the *Menus* text as a string after the menu label and the keyboard equivalent of the menu item. For example, the following entry for the *Dev* menu specifies the guard *StdCmds.SetEditModeGuard*:

    "Edit Mode"    ""    "StdCmds.SetEditMode"    "StdCmds.SetEditModeGuard"

A unique feature of menu items is that they may be checked. For example, in the following menu, menu item *Edit Mode* is checked:

Figure 4-19. Menu item with check mark

The check mark indicates which of the items has been selected most recently, and the state that has been established by the selection. The state can be changed by invoking one of the other menu items, e.g., *Mask Mode* as in the figure above. Basically, the four menu items form a group of possibilities from which one can be selected.

Guard procedures for menu items may set up the *disabled*, *checked* and *label* fields of *Dialog.Par*. The other fields are ignored for menus.

A guard procedure may set up several fields of its *par* output parameter simultaneously, e.g., it may assign the *disabled* and *label* fields for a command button. However, a guard may set at most one of the Boolean fields of *par* and must *never* modify any state outside of *par*, e.g., a field of an interactor or something else; i.e., it must have no side effects. It may not call any procedures which may have side effects either. The reason is that a program cannot assume much about when (and especially when not) a guard is being called by the framework.

A guard may use any interactor or set of interactors as its input, or the state of the current focus or selection. The latter is often used for menu guards. The current focus or selection is only used in control guards if the controls are in so-called tool dialog boxes, i.e., dialog boxes that operate on some document underneath. A *Find & Replace* dialog box operating on a focused text is a typical example of a tool dialog box. The other dialog boxes are self-contained and called auxiliary dialog boxes. Data entry masks are typical examples of auxiliary dialog boxes.

When is a guard evaluated? There are four reasons why a guard may be called: when the control's link is established, when the contents of an interactor is being edited, when the window hierarchy has changed, or when the user has clicked into the menu bar.

A control's link is established after it is newly inserted into a container, after it has been loaded from a file, after a module was unloaded, or after its link has been modified through the control property inspector or other tool.

When some piece of code modifies the contents of an interactor, it is required to call *Dialog.Update* for this interactor. As a result, every currently visible control is notified (see section 2.9). In turn, the control compares its actual state with the interactor field to which it is linked. If the interactor field has changed, the control redraws itself accordingly. After all controls have updated themselves, the guards of *all* visible controls are evaluated. For this reason, guards should be efficient and not perform too much processing.

Guards are also evaluated when the window hierarchy is changed, e.g., when a bottom window is brought to the top. This is necessary because many commands depend on the current focus or selection, which vanishes if another window comes to the top.

Menus are another reason why a guard may be called. When the user clicks in a menu bar, all guards of this menu, or even the guards of the whole menu bar, are evaluated.

Usually, a guard has the form

    PROCEDURE SomeGuard* (VAR par: Dialog.Par)

Alternatively, the form

    PROCEDURE SomeGuard* (n: INTEGER; VAR par: Dialog.Par)

may be used, which allows to parameterize a single guard procedure for several related commands. For example, the commands to set a selection to the colors red, green or blue are the following:

    StdCmds.Color(00000FFH)

    StdCmds.Color(000FF00H)

    StdCmds.Color(0FF0000H)

For these commands, the following guards can be used:

    StdCmds.ColorGuard(00000FFH)

    StdCmds.ColorGuard(000FF00H)

    StdCmds.ColorGuard(0FF0000H)

The actual signature of *StdCmds.ColorGuard* is

    PROCEDURE ColorGuard (color: INTEGER; VAR par: Dialog.Par)

**4.5 Notifiers**

A control guard sets up the temporary state of a control, such as whether it is disabled or not. It has no other effect. It doesn't modify any interactor state. It doesn't add any functionality. It only lets the control give feedback about the currently available functionality, or its lack thereof. A guard is evaluated ,e.g., when the user changes the state of a control interactively. It can be regarded as a merely "cosmetic" feature whose sole purpose is to increase user-friendliness.

However, sometimes a user interaction should trigger more than the evaluation of guards only. In particular, it may sometimes be necessary to change some interactor state as a response to user interaction. For example, changing the selection in a selection box may cause the update of a corresponding counter, which counts the number of currently selected items in the selection box. Or some postprocessing of user input may be implemented, as we will see in the following example. It is an alternate implementation of our *ObxPhoneUI* example. Instead of having a *Lookup* button, this version just has two edit fields. After any character typed in, a database lookup is executed to test whether now a correct key is entered. Note that unsuccessful lookup in *ObxPhoneDB* results in returning an empty string.

MODULE ObxPhoneUI1;

    IMPORT Dialog, ObxPhoneDB;

    VAR

        **phone***: RECORD

            name*, number*: ObxPhoneDB.String

        END;

    PROCEDURE **NameNotifier*** (op, from, to: INTEGER);

    BEGIN

        ObxPhoneDB.LookupByName(phone.name, phone.number);

        Dialog.Update(phone)

    END NameNotifier;

    PROCEDURE **NumberNotifier*** (op, from, to: INTEGER);

    BEGIN

        ObxPhoneDB.LookupByNumber(phone.number, phone.name);

        Dialog.Update(phone)

    END NumberNotifier;

END ObxPhoneUI1.

Listing 4-20. ObxPhoneUI1 with notifiers

These *notifiers* create dependencies between two fields of an interactor: a modification of one text field may lead to a modification of the other text field, i.e., a dependency between interactor fields is defined.

Notifiers are called right after an interaction happens, but before the guards are evaluated. A notifier has the following form:

    PROCEDURE XyzNotifier* (op, from, to: INTEGER);

    BEGIN

        ...

    END XyzNotifier;

A notifier's type is

    NotifierProc = PROCEDURE (op, from, to: INTEGER);

For simple notifiers, the parameters can be ignored. They give a more precise indication of what kind of modification actually took place. Which of the parameters are valid and what their meaning exactly is is defined separately for every kind of control. See the  next section for details.

**4.6 Standard controls**

In this section, the various controls that come standard with the BlackBox Component Builder are presented. For each control, a list of data types to which it may be linked is given, and the meaning of the notifier parameters is indicated.

A control may have various properties, e.g., its label, or whether the label should be displayed in the normal system font or in a lighter, less flashy font. However, such cosmetic properties may not be implemented the same way on every platform, sometimes they may even be ignored.

A label may specify a keyboard shortcut, which on some platforms is used to navigate between controls without using the mouse. The shortcut character is indicated by preceding it with a "&" sign. Two successive "&" signs indicate a normal "&", without interpreting the following character as shortcut. Only one shortcut per label is allowed. For example, "Clear Text &Field" defines "F" as a keyboard shortcut, while "Find && &Replace" defines "R" as keyboard shortcut. The same syntax for keyboard shortcuts is used in menu commands.

The label property of every control can be modified at run-time, by using a guard procedure as demonstrated in Listing 4-16. For controls without visible label, this has no visible effect.

Two of the most important control properties, the guard and notifier names, are optional. If there is no guard for the control, it will use its default states (enabled, read-write, defined). If there is no notifier for the control, none will be called.

BlackBox attempts to minimize the number of different properties, in order to make it easier to use. This is consistent with the general trend, both under Windows and the Mac OS, towards globally controlled appearances. This means that a control should not define its colors, font, and so on individually. Instead, the user should be able to define a system-wide and consistent configuration of these properties.

However, if for some special reason this is still deemed important, specific fonts can be assigned to controls, simply by selecting the control(s) and applying the commands of the *Font*, *Attributes* or *Characters* menus. If the default font is set, then the system preferences will be used. This is the normal case. Note that other font attributes, such as styles, weight, or size, may be restricted on certain platforms. For example, on Mac OS the font size is always 12 points.

In the following text, all standard controls of BlackBox are described in turn: command buttons, edit fields, check boxes, radio buttons, date fields, time fields, color fields, captions, groups, list boxes, selection boxes and combo boxes. For each control, its valid properties are given, the variable types to which it may be linked, and the meaning of the *op*, *from* and *to* parameters of the notifier.

Note that there is no special control for currency values, you can use edit fields bound to variables of type *Dialog.Currency* instead.

In the following descriptions, some abbreviations are used:

*pressed* stands for *Dialog.pressed*

*released* stands for *Dialog.released*

*changed* stands for *Dialog.changed*

*included* stands for *Dialog.included*

*excluded* stands for *Dialog.excluded*

*undefined* means that the *from* or *to* parameter of a notifier has no defined value that can be relied upon

*modifier* means a 0 for single-clicks and a 1 for double-clicks

*link*, *label*, *guard*, *notifier*, and* level* stand for their respective control properties as defined in module *Controls*. There are up to five optional Boolean properties. Depending on the control, they are called differently, e.g. *default font*, *default*, *cancel*, *sorted*, *left*, *right*, *multiLine*, *password*.

**Command Button**

Figure 4-21. Command button

A command button is used to execute a parameterless Component Pascal command, or a whole command sequence. A button with the default property looks different than normal buttons; input of a carriage return corresponds to a mouse click in the default button. For a button with the cancel property, input of an escape character corresponds to a mouse click in it. A button should not be default and cancel button simultaneously. Note that the default and cancel properties are set and cleared individually per control, i.e., making one button a default button doesn't automatically make an existing default button a normal button again. There should be at most one default and at most one cancel button per dialog box.

A cancel button or another button that should close the dialog box in which it is contained can use the command *StdCmds.CloseDialog*.

properties:    link, label, guard, notifier, font, default, cancel

linkable to:    parameterless procedure, command sequence

op:    pressed, released

from:    undefined, modifier

to:    undefined

**Text Field**

Figure 4-22. Text field

A text field displays the value of a global variable which may be a string, an integer, a floating-point number, or a variable of type *Dialog.Currency* or *Dialog.Combo*.  The value of the variable can be changed by editing the field.

Whenever the contents of the linked interactor field is changed, the notifier is called. Key presses that do not change the interactor state, such as pressing arrow keys or entering leading zeroes in number fields, cause no notifier call. Changing the selection contained in the field causes no notification either.

If a modification occurs in a field that is linked to a string, real, currency, or combo variable, the notifier with *(change, undefined, undefined)* is called. If the field is linked to an integer value, the notifier with *(change, oldvalue, newvalue)* is called.

Illegal characters, e.g., characters in a field linked to an integer, are not accepted.

A text field may have a label, even though it doesn't display it. The reason for this is that keyboard shortcuts may be defined in labels, which is useful even for edit fields.

If the *level* property is 0 (the default), or the control is not linked to a number type, or it is linked to *Dialog.Currency*, then *level* has no effect. When linked to an integer variable the *level* defines the scale factor used for displaying the number, i.e., the displayed number is the linked value divided by 10level. For example, if the current value is 42 and level is 2, then 0.42 is displayed. For variable of real type, *level* indicates the format in the following way:

*level* > 0: exponential format (scientific) with at least *level* digits in the exponent.

*level* = 0: fixpoint or floatingpoint format, depending on *x*.

*level* < 0: fixpoint format with *-level* digits after the decimal point.

The *left* and *right* properties define the adjustment mode of the field. The following combinations are possible:

    left & ~right    left adjust

    left &   right    fully adjusted (may have no effect on some platforms)

  ~left & ~right    centered

  ~left &   right    right adjust

The default is *left & ~right* (left adjust).

Property *multiLine* defines whether a carriage return, with its resulting line break, may be accepted by the field, which then must be linked to a string variable.

Property *password* causes the text field to display only asterisks instead of the characters typed in. This makes it possible to use such a field for password entry.

properties:    link, label, guard, notifier, level, font, left, right, multiLine,

    password

linkable to:    ARRAY const OF CHAR,

    BYTE, SHORTINT, INTEGER, LONGINT, SHORTREAL, REAL,

    Dialog.Currency, Dialog.Combo

op:    pressed, released, changed

from:    undefined, old value, modifier

to:    undefined, new value

**Check Box**

Figure 4-23. Check box

A check box displays the value of a global variable which may be a Boolean or an element of a set. Clicking on the control toggles its state.

When the control's state is changed, the notifier with *(changed, undefined, undefined)* is called if the control is linked to a Boolean variable. If it is linked to an element of a set variable, *(included, level, undefined)* or *(excluded, level, undefined)* is called, depending on whether the bit is set or cleared. The value *level* corresponds to the element of the set to which the control is linked. It can be defined using the control property inspector. It may lie in the range 0..31.

Only the last and final state change of the control leads to a notifier call, possible intermediate state changes (by dragging the mouse outside of the control's bounding box or back inside) have no effect.

properties:    link, label, guard, notifier, font, level

linkable to:    BOOLEAN, SET

op:    pressed, released, changed, included, excluded

from:    undefined, level value, modifier

to:    undefined

**Radio Button**

Figure 4-24. Radio button

A radio button is active at a particular value of a global integer or Boolean variable. Typically, several radio buttons are linked to the same variable. Each radio button is "on" at another value, which is defined by the level property; i.e., the button is "on" if its level value is equal to the value of the variable it is linked to. For Boolean types, "on" corresponds to *TRUE* and "off" corresponds to *FALSE*.

Only the last and final state change of the control leads to  a notifier call, possible intermediate state changes (by dragging the mouse outside of the control's bounding box or back inside) have no effect.

properties:    link, label, guard, notifier, font, level

linkable to:    BYTE, SHORTINT, INTEGER, LONGINT, BOOLEAN

op:    pressed, released, changed

from:    undefined, old value, modifier

to:    undefined, new value = level value

**Date Field**

Figure 4-25. Date field

A date field displays the date specified in a global variable of type *Dates.Date*.

Whenever the contents of the linked interactor field is changed, the notifier is called with *(change, undefined, undefined)*. Key presses that do not change the interactor state, such as pressing left/right arrow keys, cause no notifier call. The up/down arrow keys change the date. Changing the selection in the field has no effect.

Illegal date values cannot be entered.

properties:    link, label, guard, notifier, font

linkable to:    Dates.Date

op:    pressed, released, changed

from:    undefined, modifier

to:    undefined

**Time Field**

Figure 4-26. Time field

A time field displays the time specified in a global variable of type *Dates.Time*.

Whenever the contents of the linked interactor field is changed, the notifier is called with *(change, undefined, undefined)*. Key presses that do not change the interactor state, such as pressing left/right arrow keys, cause no notifier call. The up/down arrow keys change the time. Changing the selection in the field has no effect.

Illegal time values cannot be entered.

properties:    link, label, guard, notifier, font

linkable to:    Dates.Time

op:    pressed, released, changed

from:    undefined, modifier

to:    undefined

**Color Field**

Figure 4-27. Color field

A color field displays a color. It can be linked to variables of type *INTEGER* or *Dialog.Color*. *Ports.Color* is an alias of *INTEGER* and thus can be used also.

Whenever another color is selected, the notifier is called with *(change, oldval, newval)*. The old and new values are integer values; for *Dialog.Color* type variables the *val* field's value is taken.

properties:    link, label, guard, notifier, font

linkable to:    Dialog.Color, Ports.Color = INTEGER

op:    pressed, released, changed

from:    undefined, old value, modifier

to:    undefined, new value

**Up/Down Field**

Figure 4-28. Up/down field

This is a field linked to an integer variable. The value can also be changed through arrow keys.

Whenever the contents of the linked interactor field is changed, the notifier is called with *(change, oldvalue, newvalue)*.

properties:    link, label, guard, notifier, font

linkable to:    BYTE, SHORTINT, INTEGER, LONGINT

op:    pressed, released

from:    undefined, old value, modifier

to:    undefined, new value

**Caption**

Figure 4-29. Caption

A caption is typically used in conjunction with a text field, to indicate the nature of its contents. A caption is passive, i.e., it cannot be edited and thus cannot have a notifier.

A caption is linkable to the same types as a text field is, and it may have a guard. This is useful since - depending on the platform - a caption may have a distinct visual appearance if it (or rather its corresponding text field) is disabled, read-only, etc. This means that a caption may be disabled along with its corresponding text field, by linking both to the same interactor field. A caption's guard may modify the control's label, as is true for all controls with a visible label.

properties:    link, label, guard, font, right, left

linkable to:    ARRAY const OF CHAR,

    BYTE, SHORTINT, INTEGER, LONGINT, SHORTREAL, REAL,

    Dialog.Currency, Dialog.Combo

The *left* and *right* properties define the adjustment mode of the caption. The following combinations are possible:

    left & ~right    left adjust

    left &   right    fully adjusted (may have no effect on some platforms)

  ~left & ~right    centered

  ~left &   right    right adjust

The default is *left & ~right* (left adjust).

**Group**

Figure 4-30. Group

A group is used to visually group related controls, e.g., radio buttons that belong together. A group is passive, i.e., it cannot be edited and thus cannot have a notifier.

A group may not be linked, but it may have a guard which allows to disable or enable it. A group's guard may modify the control's label, as is true for all controls with a visible label.

properties:    label, guard, font

**List Box**

Figure 4-31. List box (expanded and collapsed shapes)

A list box allows to select one value out of a list of choices. It is linked to a variable of type *Dialog.List* (see next section). If the height of the list box is large enough, a scrollable list is displayed. If it is not large enough, the box collapses into a pop-up menu.

Interactively, the selection can be changed. It is either empty or it consists of one selected item. When the user modifies the selection, the notifier is called with *(changed, oldvalue, newvalue)*. The old/new value corresponds to the selected item's index. The top-most item corresponds to value 0, the next one below to value 1, etc. The empty selection corresponds to value -1.

The *sorted* property determines whether the string items will be sorted lexically (no effect on Mac OS).

properties:    link, label, guard, notifier, font, sorted

linkable to:    Dialog.List

op:    pressed, released, changed

from:    undefined, old value, modifier

to:    undefined, new value

**Selection Box**

Figure 4-32. Selection box

A selection box allows to select a subset out of a set of choices. It is linked to a variable of type *Dialog.Selection* (see next section).

Interactively, the selection can be changed. Each item may be selected individually. When the user modifies the selection, the notifier is called in one of three ways:

*(included, from, to)*: range *from..to* is now selected; it wasn't selected before

*(excluded, from, to)*: range *from..to* is not selected now; it was selected before

*(set, from, to)*: range *from..to* is now selected; any previous selection was cleared before

The three codes *included* and *excluded* and *set* take the place of *changed* used for most other controls. The notifier is called as often as necessary to include and/or exclude all necessary ranges of items.

The from/to value corresponds to the selected item's index. The topmost item corresponds to value 0, the next one below to value 1, and so on.

The *sorted* property determines whether the string items will be sorted lexically (no effect on Mac OS).

properties:    link, label, guard, notifier, font, sorted

linkable to:    Dialog.Selection

op:    pressed, released, included, excluded, set

from:    undefined, lowest element of range, modifier

to:    undefined, highest element of range

**Combo Box**

Figure 4-33. Combo box

A combo box is a text field whose contents can also be set via a pop-up menu. Unlike pure pop-up menus/selection boxes, a value may be entered which does not occur in the pop-up menu. The control is linked to a variable of type *Dialog.Combo* (see next section).

When the contents of the combo is changed, the notifier is called with *(changed, undefined, undefined)*.

The *sorted* property determines whether the string items will be sorted lexically (no effect on Mac OS).

properties:    link, label, guard, notifier, font, sorted

linkable to:    Dialog.Combo

op:    pressed, released, changed

from:    undefined, modifier

to:    undefined

Each interactive control (i.e., not a caption or a group) calls its notifier - if there is one - when the user first clicks in the control with parameter *op = Dialog.pressed*, and later with *op = Dialog.released* when the mouse button is released again. This feature is used mostly to display some string in the dialog box window's status area. This is done with the calls *Dialog.ShowStatus* or *Dialog.ShowParamStatus*. For example, the following notifier indicates a command button's function to the user:

  PROCEDURE ButtonNotifier* (op, from, to: LONGINT);

  BEGIN

    IF op = Dialog.pressed THEN

      Dialog.ShowStatus("This button causes the disk to spin down")

    ELSIF op = Dialog.released THEN

      Dialog.ShowStatus("")    *(* clear the status message again *)*

    END

  END ButtonNotifier;

Listing 4-34. Notifier displaying status messages

On some platforms, e.g., on Mac OS, there is no status area and the above code has no effect.

Sometimes it is useful to detect double-clicks, e.g., a double-click in a list box may select the item and invoke the default button. A double-click can be detected in a notifier with the test

  IF (op = Dialog.pressed) & (from = 1) THEN ...    *(* double-click *)*

For the above-mentioned case, where the reaction on a double-click should merely be the invocation of the default button, a suitable standard notifier is available:

  StdCmds.DefaultOnDoubleClick

We have seen earlier that the BlackBox Component Builder provides a string mapping facility which maps keys to actual strings, by using resource files. This feature is also supported by *Dialog.ShowStatus*. In fact, string mapping even allows to use place holders. For example, calling

  Dialog.ShowParamStatus("This ^0 causes the ^1 to ^2", control, object, verb)

allows to supply different strings for the place holders ^0, ^1 and ^2. The strings actually used are the three additional parameters *control, object* and *verb*. Note that these strings are mapped themselves before being spliced into the first string. String mapping is a feature that you can use explicitly by calling the procedure *Dialog.MapParamString*.

**4.7 Complex controls and interactors**

A list box has two kinds of contents: the string items which make up the list, and the current selection. Typically, the item list is more persistent than the selection, but it too may be changed while the control is being used. For list and selection boxes, the application is not so much interested in the item list, since this list is only a hint for the user. The application is interested in the selection that the user creates. For a list box, the selection is defined by the index of the selected item. For a selection box, the selection is defined by the set of indices of selected items. For combo boxes, the relevant state is not a selection, but the string that was entered.

For all three box controls, the item list must be built up somehow. For this purpose, module *Dialog* defines suitable interactor types: *Dialog.List* for list boxes, *Dialog.Selection* for selection boxes, and *Dialog.Combo* for combo boxes. These types are defined the following way:

    List = RECORD

        index: INTEGER;    *(* index of currently selected item *)*

        len-: INTEGER;    *(* number of list elements *)*

        PROCEDURE (VAR l: List) SetLen (len: INTEGER), NEW;

        PROCEDURE (VAR l: List) SetItem (index: INTEGER; IN item: ARRAY OF CHAR), NEW;

        PROCEDURE (VAR l: List) GetItem (index: INTEGER; OUT item: String), NEW;

        PROCEDURE (VAR l: List) SetResources (IN key: ARRAY OF CHAR), NEW

    END;

    Selection = RECORD

        len-: INTEGER;    *(* number of selection elements *)*

        PROCEDURE (VAR s: Selection) SetLen (len: INTEGER), NEW;

        PROCEDURE (VAR s: Selection) SetItem (index: INTEGER; IN item: ARRAY OF CHAR), NEW;

        PROCEDURE (VAR s: Selection) GetItem (index: INTEGER; OUT item: String), NEW;

        PROCEDURE (VAR s: Selection) SetResources (IN key: ARRAY OF CHAR), NEW;

        PROCEDURE (VAR s: Selection) Incl (from, to: INTEGER), NEW;

                                                                    *(* select range [from..to] *)*

        PROCEDURE (VAR s: Selection) Excl (from, to: INTEGER), NEW;

                                                                    *(* deselect range [from..to] *)*

        PROCEDURE (VAR s: Selection) In (index: INTEGER): BOOLEAN, NEW

                                                                    *(* test whether index-th item is selected *)*

    END;

    Combo = RECORD

        item: String;    *(* currently entered or selected string *)*

        len-: INTEGER;    *(* number of combo elements *)*

        PROCEDURE (VAR c: Combo) SetLen (len: INTEGER), NEW;

        PROCEDURE (VAR c: Combo) SetItem (index: INTEGER; IN item: ARRAY OF CHAR), NEW;

        PROCEDURE (VAR c: Combo) GetItem (index: INTEGER; OUT item: String), NEW;

        PROCEDURE (VAR c: Combo) SetResources (IN key: ARRAY OF CHAR), NEW

    END;

Listing 4-35. Definitions of List, Selection and Combo

Before a variable of one of these types can be used, it must be initialized by first defining the individual items. During use, items can be changed. For example, the following code fragment builds up a list:

    list.SetLen(5);    *(* define length of list *)*

    list.SetItem(0, "Daffy Duck");

    list.SetItem(1, "Wile E. Coyote");

    list.SetItem(2, "Scrooge Mc Duck");

    list.SetItem(3, "Huey Lewis");

    list.SetItem(4, "Thomas Dewey");

    Dialog.UpdateList(list);    *(* must be called after any change(s) to the item list *)*

Listing 4-36. Setting up a list explicitly

When used with list-structured controls (list, selection, or combo boxes), *Dialog.Update* only updates the selection or text entry state of these controls, but not the list structure. If the list structure, i.e., the elements of the control's list, is changed, then the procedure *Dialog.UpdateList* must be called instead of *Dialog.Update*.

For fixed item lists, the individual strings should be stored in resources. This is simplified by the *SetResources* procedures. They look up the strings in a resource file. For example, the above statements can be replaced completely by the statement

    list.SetResources("#Obx:list")

which will read the Obx/Rsrc/Strings file and look for strings with keys of the kind "list[index]", e.g., for

    list[0]    Daffy Duck

    list[1]    Wile E. Coyote

    list[2]    Scrooge Mc Duck

    list[3]    Huey Lewis

    list[4]    Thomas Dewey

Table 4-37. Resources for a list

The indices must start from 0 and be consecutive (no holes allowed).

Procedure *SetLen* is optional. Its use is recommended wherever the number of list items is known in advance. If this is not the case, it may be omitted. The list will become as large as the largest index requires.

*SetLen* is necessary if an existing item list should be shortened, or if it should be cleared completely.

**4.8 Input validation**

Validity checks are an important, and for the uninitiated sometimes a surprising, aspect of non-modal user interfaces. Among other things, non-modality also means that the user must not be forced into a mode depending on where the caret currently is and whether or not the entered data is currently valid. In particular, a user must not be forced to correctly enter some data into a field before permitting him or her to do something else.

This basically leaves two strategies for checking the validity of entered data: early or late. Early checks are performed whenever the user has manipulated a control. Late checks are performed when the user has completed input and wants to perform some action, e.g., entering the new data into a database.

Late checks are most suitable for checking global invariants, e.g., whether all necessary fields in the input mask contain some input. Early checks are most suitable for local, control-specific invariants, e.g., the correct syntax of an entered string.

We give examples of both early and late checks for our module *ObxPhoneUI*. Let us assume that a phone number always has the following form: 310-555-1212. For a late check, we extend the *Lookup* procedure (boldface text) and add a few auxiliary procedures. Note the use of the "$" operator, which makes sure the *LEN* function returns the length of the string, not of the array containing the string.

    PROCEDURE Valid (IN s: ARRAY OF CHAR): BOOLEAN;

        PROCEDURE Digits (IN s: ARRAY OF CHAR; from, to: INTEGER): BOOLEAN;

        BEGIN    *(* check whether range [from..to] in s consists of digits only *)*

            WHILE (from <= to) & (s[from] >= "0") & (s[from] <= "9") DO INC(from) END;

            RETURN from > to    *(* no non-digit found in checked range *)*

        END Digits;

    BEGIN    *(* check syntax of phone number *)*

        RETURN (LEN(s$) = 12) & Digits(s, 0, 2) & (s[3] = "-") & Digits(s, 4, 6) &

                            (s[7] = "-") & Digits(s, 8, 11)

    END Valid;

    PROCEDURE ShowErrorMessage;

    BEGIN

        phone.name := "illegal syntax of number"

    END ShowErrorMessage;

    PROCEDURE Lookup*;

    BEGIN

        IF phone.lookupByName THEN

            ObxPhoneDB.LookupByName(phone.name, phone.number);

            IF phone.number = "" THEN phone.number := "not found" END

        ELSE

            **IF Valid(phone.number) THEN**

                ObxPhoneDB.LookupByNumber(phone.number, phone.name);

                IF phone.name = "" THEN phone.name := "not found" END

**            ELSE**

**                ShowErrorMessage**

            **END**

        END;

        Dialog.Update(phone)

    END Lookup;

Listing 4-38. Late check for input validation

A more rude reminder would be to display an error message. If the BlackBox Component Builder's log window is open, the message is written to the log and the window is brought to the top if necessary. If no log is used, a dialog box is displayed. This behavior can be achieved by replacing the statement in procedure *ShowErrorMessage* by the following statement:

    Dialog.ShowMsg("Please correct the phone number")

Now we look at an early checked alternative to the above solution to input checking. It uses a notifier to check after each character typed into the phone number field whether the number is legal so far. In contrast to the late check, it must be able to deal with partially entered phone numbers. Whenever an illegal suffix is detected, the string is simply clipped to the legal prefix.

    PROCEDURE Correct (VAR s: ObxPhoneDB.String);

        PROCEDURE CheckMinus (VAR s: ObxPhoneDB.String; at: INTEGER);

        BEGIN

            IF s[at] # "-" THEN s[at] := 0X END    *(* clip string *)*

        END CheckMinus;

        PROCEDURE CheckDigits (VAR s: ARRAY OF CHAR; from, to: INTEGER);

        BEGIN

            WHILE from <= to DO

                IF (s[from] < "0") OR (s[from] > "9") THEN

                    s[from] := 0X; from := to    *(* clip string and terminate loop *)*

                END;

                INC(from)

            END

        END CheckDigits;

    BEGIN    *(* clip string to a legal prefix if necessary *)*

        CheckDigits(s, 0, 2);

        CheckMinus(s, 3);

        CheckDigits(s, 4, 6);

        CheckMinus(s, 7);

        CheckDigits(s, 8, 11)

    END Correct;

    PROCEDURE NumberNotifier (op, from, to: INTEGER);

    BEGIN

        Correct(phone.number)

        *(* *Dialog.Update* is not called, because it will be called afterwards by the notifying control *)*

    END NumberNotifier;

Listing 4-39. Early check for input validation

Note that a focused view, e.g. a control, has no means to prevent the user from focusing another view. A view may merely change the way that it displays itself, its contents, or its marks (selection, caret). For this purpose, a view receives a *Controllers.MarkMsg* when the focus changes.

**4.9 Accessing controls explicitly**

In most circumstances, guards and notifiers allow sufficient control over a control's specific look & feel. However, sometimes you may want to exercise direct control over a control in a form. How to do this is described in this section. For example, assume that you want to write your own special alignment command, and add it to the commands of the *Layout* menu. To do this, you need to get access to the form view in the window that is currently being edited. The form view is a container that contains the controls that you want to manipulate. The function *FormControllers.Focus* delivers a handle on the currently focused form editor. This leads to the typical code pattern for accessing a form (Listing 4-40):

        VAR c: FormControllers.Controller;

    BEGIN

        c := FormControllers.Focus();

        IF c # NIL THEN

            ...

Listing 4-40. Accessing a form's controller

A form contains controls and possibly some other types of views. The views can be edited if the enclosing form view is in layout mode. Typically, some views are selected first, and then a command is executed that operates on the selected views. The following example shifts every selected view to the right by one centimeter (Listing 4-41):

MODULE ObxControlShifter;

    IMPORT Ports, Views, FormModels, FormControllers;

    PROCEDURE **Shift***;

        VAR c: FormControllers.Controller; sel: FormControllers.List;

    BEGIN

        c := FormControllers.Focus();

        IF (c # NIL) & c.HasSelection() THEN

            sel := c.GetSelection();    *(* generates a list with references to the selected views *)*

            WHILE sel # NIL DO

                c.form.Move(sel.view, 10 * Ports.mm, 0);    *(* move to the right *)*

                sel := sel.next

            END

        END

    END Shift;

END ObxControlShifter.

Listing 4-41. Shift all selected views to the right

This code works on selected views (whether they are controls or not). Sometimes you may want to manipulate views that are not selected. In this case, you need a form reader (*FormModels.Reader*) to iterate over the views in the currently focused form. The following code pattern shows how a command can iterate over the views of a form (Listing 4-42):

        VAR c: FormControllers.Controller; rd: FormModels.Reader;

    BEGIN

        c := FormControllers.Focus();

        IF c # NIL THEN

            rd := c.form.NewReader(NIL);

            rd.ReadView(v);    *(* read first view *)*

            WHILE v # NIL DO

                ...

                rd.ReadView(v)    *(* read next view *)*

            END;

            ...

Listing 4-42. Iterating over the selected views in a form

Controls are special views (of type *Controls.Control*). Controls can be linked to global variables. The following example shows how the labels of all controls in a form can be listed (Listing 4-43):

MODULE ObxLabelLister;

    IMPORT Views, Controls, FormModels, FormControllers, StdLog;

    PROCEDURE **List***;

        VAR c: FormControllers.Controller; rd: FormModels.Reader; v: Views.View;

    BEGIN

        c := FormControllers.Focus();

        IF c # NIL THEN

            rd := c.form.NewReader(NIL);

            rd.ReadView(v);    *(* read first view *)*

            WHILE v # NIL DO

                IF v IS Controls.Control THEN

                    StdLog.String(v(Controls.Control).label); StdLog.Ln

                END;

                rd.ReadView(v)    *(* read next view *)*

            END

        END

    END List;

END ObxLabelLister.

Listing 4-43. Listing the labels of all controls in a form

With the same kind of iteration, any other type of view could be found in forms as well, not only controls. For example, consider that you have a form with a "Clear" command button that causes a plotter view in the same form to be cleared. The command button is a standard control, the plotter view is your special view type. The command associated with the command button first needs to search for its plotter view, in the same form where the button is placed. The code pattern shown below demonstrates how such a command can be implemented (Listing 4-44):

        VAR c: FormControllers.Controller; rd: FormModels.Reader; v: Views.View;

    BEGIN

        c := FormControllers.Focus();

        IF c # NIL THEN

            rd := c.form.NewReader(NIL);

            rd.ReadView(v);    *(* read first view *)*

            WHILE v # NIL DO

                IF v IS MyPlotterView THEN

                    *(* clear v(MyPlotterView) *)*

                END;

                rd.ReadView(v)    *(* read next view *)*

            END

        END

Listing 4-44. Finding a particular view in a form

There is one potential problem with the above code pattern. Let us assume that we have a dialog box containing a command button with *ObxLabelLister.List* as associated command. Beneath the dialog box, we have a window with a focused form layout. Now, if you click on the dialog box' command button, which labels are listed? The ones in the form of the dialog box itself, or the ones in the focused form layout underneath? In other words: does *FormControllers.Focus* yield the form that contains the button you clicked on, or the form focused for editing? Depending on what the button's command does, both versions could make sense. For a form editing command like *ObxControlShifter.Shift*, the focused layout editor should be returned. This is the top-most document window, but it is overlaid by the dialog box window. In contrast, if you want to find your plotter view, the form of the dialog box should be returned instead. In this case, you want to search the direct "neighborhood" of the button.

One solution for the plotter view search would be to start searching in the context of the button that is currently being pressed. This can be done using the following code pattern (Listing 4-45):

        VAR button: Controls.Control; m: Models.Model; rd: FormModels.Reader; v: Views.View;

    BEGIN

        button := Controls.par;    *(* during the button click, this variable contains a reference to the button *)*

        m := button.context.ThisModel();    *(* get the model of the container view that contains the button *)*

        IF m IS FormModels.Model THEN    *(* the container is a form *)*

            rd := m(FormModels.Model).NewReader(NIL);

            rd.ReadView(v);    *(* read first view *)*

            WHILE v # NIL DO

                IF v IS MyPlotterView THEN

                    *(* clear v(MyPlotterView) *)*

                END;

                rd.ReadView(v)    *(* read next view *)*

            END

        END

Listing 4-45. Finding a particular view in the same form as a button

This code assumes that the command is executed from a command button. It doesn't work if placed in a menu. In order to better decouple the user interface and the application logic, this assumption should be eliminated. BlackBox solves the problem by giving the programmer explicit control over the behavior of *FormControllers.Focus*. The trick is that BlackBox supports two kind of windows for dialog boxes: *tool windows* and *auxiliary windows, *in addition to the normal document windows. If the command button is in a tool window dialog box, then *FormControllers.Focus* yields the form in the top-most *document* window, or *NIL* if there is no such document window or if the focus of the top-most document window is not a form view. If the command button is in an auxiliary window, then *FormControllers.Focus* yields the dialog box' form whose command button you have clicked. The following paragraphs further explain the differences between these two kinds of windows.

Dialog boxes in tool windows are not self-contained, they provide the parameters and the control panel for an operation on a document underneath. A typical example is the "Find / Replace" dialog box that operates on a text document underneath. The command buttons in a tool dialog box invoke a command, and this command fetches the view on which it operates. In the "Find / Replace" example, this is the focused text view. Under Windows, tool windows always lie above the topmost document window; i.e., they can never be overlapped by document windows or auxiliary windows, only by other tool windows. Unlike other windows, tool windows cannot be resized or iconized, but can me moved outside of the application window. On Mac OS, tool windows look and behave like document windows, except that they have no scroll bars and cannot be resized or zoomed.

An auxiliary window on the other hand is self-contained. It contains, and operates on, its own data. At least, it knows where to find the data on which to operate (e.g., in a database). The "Phone Database" dialog box is a typical auxiliary window, like most data-entry forms. Auxiliary windows can be manipulated like normal document windows; in particular, the may be overlapped by other document or auxiliary windows.

Both kinds of dialog boxes are stored as documents in their appropriate *RSRC* directories. But to open them, module *StdCmds* provides two separate commands: *OpenToolDialog* and *OpenAuxDialog*. Both of them take a portable path name of the resource document as first parameter, and the title of the dialog box window as a second parameter. For example, the following menu commands may be used:

    "Find / Replace..."    ""    "StdCmds.OpenToolDialog('Text/Rsrc/Cmds', 'Find / Replace')"    ""

    "Phone Database..."    ""    "StdCmds.OpenAuxDialog('Obx/Rsrc/PhoneUI', 'Phone Database')"    ""

The function *FormControllers.Focus* returns different results, depending on whether the form is in a tool or in an auxiliary window.

For more examples on how forms can be manipulated, see also module *FormCmds*. It is available in source form, like the rest of the *Form* subsystem.

**4.10 Summary**

In this chapter, we have discussed the important aspects of how a graphical user interface and its supporting code is implemented in the BlackBox Component Builder. There are three parts of an application: application logic, user interface logic, and user interface resources, i.e., documents. A clean separation of these parts makes a program easier to understand, maintain, and extend.

For more examples of how the form system and controls can be used, see the on-line examples *ObxAddress0*, *ObxAddress1*, *ObxAddress2*, *ObxOrders*, *ObxControls*, *ObxDialog, *and *ObxUnitConv*. For advanced programmers, the sources of the complete *Form* subsystem may be interesting.

For more information on how to use the BlackBox development environment, consult the documentation of the following modules:

    DevCompiler, DevDebug, DevBrowser, DevInspector, DevReferences,

    DevMarkers, DevCmds, StdCmds, StdMenuTool, StdLog, FormCmds, TextCmds

To obtain more information on a module, select a module name and then invoke *Info->Documentation*.

In order to provide a straight-forward description, we only used the most important commands in this book. However, there are other useful commands in the menus *Info*, *Dev*, *Controls*, *Tools*, and *Layout*. For example, there are several "wizards" for the creation of new source code skeletons. Consult the files *System/Docu/User-Man*, *Text/Docu/User-Man*, *Form/Docu/User-Man*, and *Dev/Docu/User-Man* for a comprehensive user manual on how to use the framework, the editor (*Text* subsystem), the visual designer (*Form* subsystem), and the development tools (*Dev* subsystem).

