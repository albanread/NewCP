**CtlWord9**

*CtlWord9 *contains the automation interface for Microsoft Word 9.0. The objects contained in this module are explained in the corresponding help file (VBAWRD9.CHM) which is located on the MS Office CD but is not installed by default.

For more information about automation controllers in BlackBox see the [<u>Ctl Docu</u>](Dev-Man.odc.md).

Since all calls to the CtlWord9 interface go across the border of the process, they are not as efficient as ordenary calls in BlackBox. They can be as much as 50.000 times slower.

**Getting Started**

The interface of CtlWord9 is quite big, but generally not difficult to understand. To start with, there are only a few Objects Types that need to be known. Look at the Interface of CtlWord9 for more.

CtlWord9.Application

Represents the Word application. A new application can be obtained by calling **NewApplication()** which starts a new Word application and returns a application object. Windows might reuse this application when the user starts Word himself. Therefore closing an application object might also close documents of the user, while not closing it leaves a unused Word application in the memory. It is recommended never to open or close applications by yourself. Use the following procedures instead to get an Application object:

    VAR

        app: CtlWord9.Application;

        connect_doc: CtlWord9.Document;

    PROCEDURE Connect;

    BEGIN

        connect_doc := CtlWord9.NewDocument();

        connect_doc.Windows().Item(CtlT.Int(1)).PUTVisible(FALSE);

        app := connect_doc.Application()

    END Connect;

    PROCEDURE Disconnect;

    BEGIN

        connect_doc.Close(NIL, NIL, NIL); app := NIL

    END Disconnect;

CtlWord9.Document

Represents a Word document. Such a document can be obtained by calling the methods

**app.ActiveDocument()**, **app.Documents().Item(CtlT.Int(1))**, **app.Documents().Add(NIL, NIL, NIL, NIL)** and **app.Documents().Open(CtlT.Str(filename), NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL) **where app is an application object, or simply by calling **CtlWord9.NewDocument()**.

CtlWord9.Range

Represents a part of the content of a document. It can be obtained in various ways from a document. The easiest way is to use the method **CtlWord9.Document.Range(CtlT.Int(beg),CtlT.Int(end))** to access a part of the document, or **CtlWord9.Document.Content()** to get its whole content.

To edit the text of a Range, the methods **CtlWord9.Range.Text()** and **CtlWord9.Range.PUTText()** can be used. The location of the Range can be moved by various **CtlWord9.Range.MoveXXX** methods, the **CtlWord9.Range.Next** method or by setting the start and the end point to a different value with **CtlWord9.Range.PUTStart** and **CtlWord9.Range.PUTEnd**.

CtlWord9.Font

Represents a font of a text. It can be obtained from a CtlWord9.Range object by calling the **CtlWord9.Range.Font()** method. Attention: Since the font of a Range might not be homogeneous, not all properties might be defined.

A good starting point to use CtlWord9 is also the [<u>ObxWordEdit Example</u>](../../Obx/Docu/WordEdit.odc.md).

**Stumbling Blocks**

Here are some things we found out using Word with this interface and which one might expect to be differently.

- The **CtlWord9.NewApplication()** method always creates a new application. If the user also opens a new Word application and there is no Word running, the user gets the SAME application as BlackBox. Never expect that you are the only one using the application. If you documents are invisible, the user sees that there are other documents open in the Window menu, but he is not able to access them.

- The **CtlWord9.NewDocument()** method creates a new document in the oldest running Word. It has the same effect as if the user chooses Word from the Start menu. If no Word is not running, one is started. If this Document is closed and there is no other open document in that application, the application might also terminated. However, if the application has been created with NewApplication, the application also remains running after the last document has been closed.

- A Word document always **ends with a new-line** (0DX). If a character is inserted after this mark, Word will insert a new 0DX after that character. If the method **Collapse(CtlT.Int(CtlWord9.wdCollapseEnd))** is called for a Range which includes this ending character, the Range will be collapsed BEFORE this character. To avoid problems, this character should not be regarded part of the content.

- All the **Collection objects** are indexed starting at 1. A collection object is a object that has a name ending in 's' and which has the procedures **Item** and **_NewEnum**. However the **CtlWord9.Range** objects start indexing at 0. Range(CtlT.Int(2), CtlT.Int(4)) for example returns a range of length 2, starting at the 3rd character.

- **Languages objects** are also collections, but they cannot be indexed using numbers. The only way to access languages is by the language's name, if this is known, or by creating an enumeration object.

- BlackBox has more font available in the **Fonts.dir.TypefaceList()** than Word in **CtlWord9.Application.FontNames()**.

- The properties of a **CtlWord9.Font** are possibly undefined. Then, the constant CtlWord9.wdUndefined is used. When the property is set, it has the value -1, and when it is not set, 0. This is not documented in the MSDN. The underline property has different values, according to the CtlWord9.wdUnderlineXXX constants. The CtlWord9.Font.Name() returns an empty string, if there is no typeface defined.

- A **CtlWord9.Language** can have a dictionary installed. Then the method ActiveSpellingDictionary() returns a dictionary. Otherwise Word might ask if it should install the dictionary for some languages.

- The method **CtlWord9.Range.Delete(NIL, NIL)** deletes the content of the Range, if it is not empty. however if the content is empty (collapsed), it deletes the character to the right of the Range. Therefore Delete has a similar effect as pressing the delete key. If this is not desirable, use the method **CtlWord9.Range.PUTText(''), **which does nothing if the content of the Range is empty.

- The unit **CtlWord9.wdCharacterFormatting** can be used to access the runs of a text. A run is a homogeneous part of the text. However, we have not found a method which would accept this unit. We have not found a way to get runs from Word.

- In Word, every window and the application have a **visible property**. A window is only visible if both, the property of the document and the application are set to TRUE. Changing the property of the application also changes the properties of all the windows.

