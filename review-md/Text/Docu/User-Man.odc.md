**Text Subsystem**

**User Manual**

**Contents**

[<u>1 Creating, opening, closing, saving, and printing text documents (Windows only)</u>](#Creating Windows)

[<u>2 Creating, opening, closing, saving, and printing text documents (Mac OS only)</u>](#Creating Mac OS)

[<u>3 Basic editing</u>](#Basic Editing)

[<u>4 Navigation keys</u>](#Navigation Keys)

[<u>5 Drag & drop</u>](#Drag & Drop)

[<u>6 Find and replace</u>](#Find and)

[<u>7 Attributes</u>](#Attributes)

[<u>8 Drag & pick</u>](#Drag & Pick)

[<u>9 Text views as containers</u>](#Text Views)

[<u>10 Text setting</u>](#Text Setting)

[<u>11 Text rulers</u>](#Text Rulers)

[<u>12 Texts as information environments</u>](#Texts as)

[<u>13 Editor and browser modes</u>](#Editor and)

[<u>14 Summary of keyboard shortcuts</u>](#Summary of)

[<u>15 Windows 95 user interface guidelines (Windows only)</u>](#New User)

[<u>16 Text preferences (Windows only)</u>](#Text Preferences)

The text subsystem is intended to support tasks of program writing and documentation, but also provides basic abstractions that can be used by every application which needs editable texts or rich text output (e.g., report generators). For this reason, the text system is part of the standard distribution of the BlackBox Component Builder

It is not intended to cover all subtle features of a standard word processing application. However, the text subsystem can be used for many tasks that would usually ask for a word processor. Furthermore, the unique extensibility of the BlackBox Component Framework allows for customization and extension of the text system with few principal limitations.

The following sections cover the user interface of the text system, not its [<u>application programming interface</u>](Sys-Map.odc.md).

<a id="Creating Windows"></a>**1 Creating, opening, closing, saving, and printing text documents (Windows only)**

As for other applications, the *File* menu is used for these purposes. To open a document use *File->Open...*. The list box *Files of Type* in the *File->Open...* dialog box lets you choose between various file types. Files other than BlackBox documents are converted when they are opened. Currently, converters for plain texts (.txt), unicode texts (.utf), BlackBox symbol files (.osf), BlackBox code files (.ocf), and ETH-Oberon V2 and V4 text files are included. If the converter is both an importer and an exporter (-> Converters), the same converter used for importing will be used as default for exporting, when saving the document. For example, a text file will be written back as a text file again, not as a BlackBox document.

To create a new document, the command *File->New* can be used. It opens a window containing an editable text. Custom menu commands may allow to open windows with other contents than text. For an introduction on how to edit the menu configuration see section "Menu Configuration" in the base system's [<u>user manual</u>](../../System/Docu/User-Man.odc.md).

Saving a document works the same way as in standard Windows applications. The *Save as Type* list box indicates the converter to be used when saving the file. For example, a text may be saved as a BlackBox document or as a plain text file (.txt).

In the *File->Page Setup...* dialog box, various parameters can be set up, e.g., the currently selected paper size, a magnification factor, etc. These parameters depend on the current printer. In addition to them, the left, top, right, and bottom margins of a page can be set up. These values denote the distances from the respective paper borders. Furthermore, a standard header (off by default) can be switched on. This standard header consists of the current date and the page number.

<a id="Creating Mac OS"></a>**2 Creating, opening, closing, saving, and printing text documents (Mac OS only)**

As for other applications, the *File* menu is used for these purposes. To open a document use *File->Open...*. It will show all BlackBox documents, as well as directories and disks. If you click on the *More files* check box, all the files which can be converted to BlackBox will be displayed as well. If there are several converters applicable to a given file, the appropriate one can be selected through the *Format* pop-up menu. Currently, converters for Mac OS plain texts (TEXT), Mac OS pictures (PICT), BlackBox symbol file (oOSF), BlackBox code files (oOCF), and ETH Oberon V2 and V4 text files are installed. If the converter is both an importer and an exporter (-> Converters), the same converter used for importing will be used as default for exporting, when saving the document. For example, a TEXT file will be written back as a TEXT file again, not as a BlackBox document.

To create a new document, the command *File->New* can be used. It opens a window containing an editable text. Custom menu commands may allow to open windows with other contents than text. For an introduction on how to edit the menu configuration see section "Menu Configuration" in the base system's [<u>user manual</u>](../../System/Docu/User-Man.odc.md).

Saving a document works the same way as in standard Mac OS applications. In the *File->Save As...* dialog box, the *Stationery* check box indicates whether the document should be written as a stationery pad. By default, this check box is unchecked. The *Format* pop-up menu indicates the converter to be used when saving the file. For example, a text may be saved as BlackBox document or as a TEXT (ASCII) file.

In the *File->Page Setup...* dialog box, various parameters can be set up, e.g., the currently selected paper size, a magnification factor, etc. These parameters depend on the current printer. In addition to them, the left, top, right, and bottom margins of a page can be set up. These values denote the distances from the respective paper borders. Furthermore, a standard header (off by default) can be switched on. This standard header consists of the current date and the page number.

<a id="Basic Editing"></a>**3 Basic editing**

BlackBox text views either display a selection or an insertion point (the *caret*). Text stretches are selected by clicking at one end of the stretch and then moving the cursor over the stretch to the other end. Words may be selected by double-clicking them. Double clicking outside of a text view to the left or right of a line can be used to select whole lines. (On Mac OS, this does not work for embedded text views.) Also, an existing selection may be extended or reduced by holding down the *shift* key when clicking. Typed text appears at the caret if there is one, otherwise it replaces the selection. The *delete* key either clears the selected text, or deletes the character at the caret position.

The attributes of selected text can be modified using the *Attributes* menu (and the *Font* menu under the Mac OS). In addition to these system attributes, texts also support vertical offsets of individual characters. These can be manipulated using the commands *Superscript* and *Subscript* in menu *Text*.

Sometimes it is useful to view two or more portions of the same text simultaneously in different windows, e.g., to make a large selection by setting the caret in one view to the start of the stretch to be selected and then *shift*-clicking at the end of that stretch in the other view.

Windows:

Use the *Window->New Window* command to open an additional window which shows the same document as the front window.

Mac OS:

The *Edit->View in Window* command opens another view onto the so-called *focus view*, in a new window. The focus view is the part of a document which contains the selection or the caret, if there is one. In other words: the focus view is the currently active editor.

<a id="Navigation Keys"></a>**4 Navigation keys**

Arrow keys can be used to move the caret to the left, right, upwards, or downwards. If the *modifier* key is pressed before the left/right arrow key, the caret is moved by word, not by character. If the *modifier* key is pressed before the up/down arrow key, the caret is moved by paragraph, not by line.

If the *shift* key is used in combination with any navigation key, the current selection (or caret) is extended or shrunk accordingly. *modifier* and *shift* keys may be used together as well.

Windows:

The *Page Up* and *Page Down* keys move the caret one page up or down. If used with the *modifier* key, the caret is moved one page left or right. Finally, the *Home* and *End* keys are used to move the caret to the beginning or end of the current line, or to the beginning or end of the document, if combined with the *modifier* key.

The navigation keys can be used to scroll the document *without moving the caret* by activating the *Scroll Lock* option on the keyboard.

Mac OS:

The navigation keys on the extended keyboards (*Page up/down*, *Home*, *End*) may be used to scroll by one page at the time, or to the beginning/end of a document, respectively.

<a id="Drag &amp; Drop"></a>**5 Drag & drop**

When clicking into an already selected text stretch without moving the cursor out of that selection, the selection is removed and the caret set instead. However, by moving the cursor out of the text selection while still holding down the mouse button, the selected text stretch is dragged to another place. When releasing the mouse button over a suitable position, the selected stretch is dropped.

Windows:

If the target is not able to consume a text stretch at that position, the cursor changes to a stop sign and nothing happens. To cancel a drop operation, move the cursor to a location where the stop sign appears, or press the *esc* key.

By dragging a piece of text and dropping it to a new place, the text piece is moved to the new place. By holding down the *modifier* key when the mouse button is released, the dropped piece becomes a copy of the dragged one: a *copy* of the selected piece is inserted at the new place. A copy can also be achieved by pressing the right mouse button as a modifier of the drag & drop operation.

Drag & drop works across applications, i.e., between BlackBox and any other application which supports OLE drag & drop. If the other application understands RTF (Rich Text Format), then the text attributes are retained during copying.

Mac OS:

If the target is not able to consume a text stretch at that position, e.g., in a window's scrollbar, nothing happens. To cancel a drop operation, move the cursor into a scrollbar, window title bar, or the menu bar.

By dragging a piece of text and dropping it to a new place, the text piece is moved to the new place if drag & drop occurs within the same document, otherwise the text piece is copied. By holding down the *modifier* key when the mouse button is released, the dropped piece is a copy of the dragged one: a *copy* of the selected piece is inserted at the new place. When crossing document boundaries the dragged object is* copied*, even if the *modifier* key is not held down. To force a move across document boundaries, hold down the *control* key before releasing the mouse button.

Drag & drop works within BlackBox only, not with the Finder or other applications.

If you move the cursor over a partially obscured window while dragging, the window comes to the top when you don't move the cursor for a second or two. When dropping, the window containing the drop destination comes to the top, if it isn't there already.

Drag & drop of a singleton (see below) result in inserting the whole singleton as a view. No merge occurs. For example, a text view singleton which is dropped into another text is not merged, but inserted as a view.

<a id="Find and"></a>**6 Find and replace**

The command *Text->Find & Replace...* opens a dialog box that allows to find and replace text strings. Text may be found in a case sensitive or insensitive way (check box *Ignore case*), and the search pattern may be restricted to only match word beginnings, word endings, or both (check boxes *Word begins with* and *Word ends with*). The latter case limits the search to exact word matches.

When no text is selected there is a button called *Replace All*. This replaces all matching fragments from the start of the document to the end of it. When a selection exists, the button is labeled *Repl. All In Sel*. and the command only replaces matching fragments within the selection.

All replacing actions, including *Replace All*, can be undone. The replacement always adopts the text attributes of the (first character of the) replaced fragment.

*Text->Find Again* searches for the currently selected string, starting from the end of the selection. If there is only a caret, it searches for the previously used string starting at the caret position. Thus *Text->Find Again* can be applied even without ever using the *Text->Find & Replace...* dialog box.

*Text->Find First* searches for the currently selected string, starting from the beginning of the text.

If a string is not found from the current position to the end of the text, a beep sound is emitted. If the same operation is executed again, the search is started at the beginning of the text.

See also module [<u>TextCmds</u>](Cmds.odc.md).

<a id="Attributes"></a>**7 Attributes**

So far, the discussed features have been mainstream features of programming language editors. In this and the following chapters, more advanced features are described, which allow to use the BlackBox text subsystem also for writing documentation and other texts that require basic word processing functions.

A character of a text has several attributes: font, color, and vertical offset. The font attribute consists of the font's typeface (e.g., Times), its size (e.g., 9 point), its weight (e.g., **bold**), and its style (e.g., *italic*, <u>underlined</u>, or strikeout). Vertical offsets such as superscripts or subscripts can be selected with the *Text->Superscript* and *Text->Subscript* commands, the other attributes can be selected with the *Attributes* menu (and the *Font* menu on Mac OS).

There is a default font typeface and a default color. These values should be used for texts that have no predetermined typeface or color. Typically, the default color is black and the default typeface is Arial (Windows) / Helvetica (Mac OS). All on-line documentation uses these default values. The user can select a text stretch, give it a particular color or typeface, and then set the default color / typeface to the selection's current value. All text stretches which use the default color / typeface will then automatically take on the same values. The default typeface is particularly useful for cross-platform documents, because the user on each platform can decide which typeface is most suitable. For example, some programmers on the Macintosh prefer the Geneva typeface.

The text system uses default attributes where no specific attributes have been selected. To change the default attributes of a document, select a text stretch which has the desired attributes, and then execute *Text->Make Default Attributes*. This allows (even for empty text documents) to set up template texts ("stationeries" on Mac OS) with the right attributes.

Attributes can be copied and pasted again, using the commands *Edit -> Copy Properties* and *Edit -> Paste Properties*.

<a id="Drag &amp; Pick"></a>**8 Drag & pick**

A novel feature of the BlackBox Component Framework is the capability to drag a selected object to another place in order to *pick up attributes*. The text system uses this facility to support setting the text attributes of a selected piece of text to those used by any other visible text stretch. To drag & pick, hold down the *alt* key (Windows) / *command* key (Mac OS) while you start dragging. Then move the cursor to a similar object with the desired attributes. After releasing the button, the attributes of the selected object(s) are overwritten by the corresponding attributes of the object where the button was released. Like drag & drop, drag & pick also works across windows.

<a id="Text Views"></a>**9 Text views as containers**

A text may contain arbitrary BlackBox views, directly floating in the stream of characters. For example, the current time is displayed by a standard clock view: .

Not only simple and small views like the above clock may float in a text; any view is possible. About thirty lines below you can see a text view embedded in this text; the embedded text view itself contains two other text views.

Windows:

To see where there are embedded views in a window, you can click anywhere in white space while holding down the *alt* key.

Mac OS:

To see where there are embedded views in a window, you can click in the window's title bar while holding down the *modifier * key.

A *container view* is a view which may contain arbitrary other views; text views are examples of container views. A container view contains *intrinsic contents* (in this case, text pieces), and views. Both intrinsic contents and views may be selected, e.g., if you are reading this text on-line, you may select all its contents, including all the text and the various views floating in it (try *Edit->Select All*). However, at times you may want to select exactly one view, which is then called a *singleton*. Singletons show in a distinguished way that they are selected: with an outline around the view, and with *handles* if the view is resizable.

The view which contains the current selection, or the caret, is called the current *focus*. Except for the outermost view, the focus and all the views in which it is contained show *focus borders. *Note that a focus is *not* a selection, rather it *contains* the selection (if there is any).

Windows:

Focus borders consist of a hatched frame around a view. A focus can be turned into a singleton by pressing the *esc* key. If the view contains a selection, pressing *esc* removes the selection, thus a second press of *esc* is necessary to turn the focus into a singleton. *Shift-esc* can be used to defocus with a single key press.

Mac OS:

Focus borders consist of two grey or dotted outlines around a view. A focus can be turned into a singleton by clicking into its focus border.

If several views are nested, the user can focus the innermost simply by clicking into it. This mechanism of directly clicking into the contents of an embedded view is called *inside-out activation*. It is especially handy when dealing with views nested several levels deep:.

Of course, sometimes it is necessary to edit the text surrounding an embedded view. For example, selecting the embedded view *itself*, instead of selecting something *inside* the embedded view, makes it necessary to disable inside-out activation. This can be done by holding down the *alt* key (Windows) / *command* key (Mac OS) when clicking. To select the outermost text view embedded in a document, use the command *Edit->Select Document*. Note that this is different from *Edit->Select All*, which merely selects the focus' contents. In a container, the focusing of embedded views can be prevented altogether, by using the *Dev->Layout Mode* command. This can be convenient in a graphical container which is meant for layout editing, e.g., in the forms editor used for dialog box layouts.

An embedded view can be resized if the container allows it. To resize such a view, it needs to be selected first, as a singleton. Once it is selected, graphical handles appear that can be dragged to interactively resize the view. The view may enforce constraints on legal sizes - this is immediately visible while resizing the view. By holding down the *shift* key when resizing, two opposite handles can be dragged simultaneously, effectively turning the resize into a move in the area of the containing view. (For some containers, such as text, free moving does not make sense. In this case the move operation will have no visible effect.)

To scroll a focused embedded view, hold down the *modifier* key when the cursor is over a scroll bar.

Mac OS:

In the following text, the Windows command *Edit->Paste Object* corresponds to the Mac OS command *Edit->Paste As Part*.

Note that the clipboard supports two different paste operations, namely *Edit->Paste* and *Edit->Paste Object*. While *Edit->Paste* tries to merge the clipped *model* (the data structure displayed by the view in the clipboard) into the destination, *Edit->Paste Object* always pastes the entire view, creating an embedded view. *Edit->Paste* succeeds in merging clipped and destination model, if both are of the same kind. (For example, a text stretch copied into the clipboard is actually carried by a text view held by the clipboard. When pasting into another text, the text view in the clipboard is ignored and the clipped text is directly inserted into the destination text.) If on the other hand the models of clipboard and focus are incompatible, *Edit->Paste*<u> </u>operates the same way as *Edit->Paste Object*.

Drag & drop always follows the semantics of *Edit->Paste*, i.e., it tries to merge the dragged object into the drop target. (This is the common case when moving model pieces around *within* a view.) In order to avoid merging of objects, the clipboard can be used: *Edit->Cut* and *Edit->Paste Object* or *Edit->Copy* and *Edit->Paste Object* instead of drag & drop.

Windows:

In addition to *Edit->Paste Object*, the command *Edit->Paste to Window* can be used to open a new document containing a copy of the view currently in the clipboard.

<a id="Text Setting"></a>**10 Text setting**

A text may contain plain characters, embedded views, and various control characters. Control characters and text-aware views affect the way a text stretch is set into a text view: a TAB (inserted using the *tab* key) forces the next word to the next tab stop, a LINE (inserted using the *return* or *enter* key) ends a line and continues setting on the next one, a PARA (inserted using command *Text->New Paragraph*) ends a line and at the same time a paragraph, causing begin-of-paragraph formats to be applied to the next line. If a TAB is entered in a text for which there is no tab stop to the right of the caret, the TAB acts as a fixed-width space.

TABs at the beginning of a line can be used to control the indentation of structured text, such as programs. After a LINE is entered and the previous line has started with TABs, the new line will start with the same number of tabs (auto indentation). The commands *Text->Shift Left* and *Text->Shift Right* modify the indentation of a selected range of lines. Note that these indentation aids only work if you use TABs for indentation, but not spaces.

Special hyphens can be used to control the breaking of words. A standard hyphen, such as in Standard-Hyphen (inserted using *modifier-minus*) allows for word breaking, just as a soft-hypen (inserted using command *Text->Insert Soft-Hyphen*) does. While standard hyphens are always visible, soft-hyphens are only displayed when actually activated to break a word at the end of a line, or if marks are displayed (*Text->Show Marks* / *Hide Marks*). Non-breaking hyphens as in "Non-Breaking-Hyphen" (inserted using command *Text->Insert Non-Brk Hyphen*) prevent breaking a word after the hyphen. Non-breaking spaces (inserted using *Text->Insert Non-Brk Space*) prevent breaking words and must be used for spaces which should be underlined. A digit space is the same as a non-breaking space but with a width equal to the digit "0" of the same font. Correctly sized digit spaces are not available for all fonts.

<a id="Text Rulers"></a>**11 Text rulers**

The most prominent text-aware views that affect text setting are *TextRulers*. Every text view contains an invisible default ruler that controls the setting of text in the absence of other rulers. A new ruler can be inserted using command *Text->Insert Ruler*. Rulers and PARA characters are usually invisible; the command *Text->Show Marks* / *Hide Marks* can be used to make both visible.

A ruler has two active (clickable) areas: an icon bar at the top, and a tab stop and margin marking bar at the bottom. A passive scale is displayed in the middle.

The leftmost icon allows to switch a right margin on (triangle icon) or off (empty rectangle icon). If the right margin is on, text setting is determined by this margin, i.e., line breaking occurs there, independent of the text view's size. This means that a view may be larger or smaller than the displayed text. In the latter case, part of the text is clipped away. If the right margin is off, lines are broken at the view border's right side, thus automatically adapting to a change of the view's width. For the outermost view, its width is determined by the page setup. Thus it typically is best to have the right margin switched off.

At the left end there are three icons to adjust the line grid, where clicking the left icon decrements, and clicking the right icon increments, the line grid. Between these two icons the actual line grid setting is shown in points; a value of 0 signals that the line grid has been disabled. This icon can be selected to set the line grid directly using the size entries in the *Attributes* menu. A double-click on the icon opens a *Size* dialog box. By default, the line grid is disabled (set to 0).

Note that enabling a line grid can have unexpected results, especially if you successively increment or decrement it by clicking on the line grid icons. Since lines are always forced to lie on the grid, without overlapping each other, small changes of the line grid or of the font size may force a line to jump one whole line up or down.

The next four icons allow to set the formatting mode. Possible modes are: flush left, centered, flush right, and fully justified. The current mode's icon is highlighted. The next icon triple allows adjustment of the lead space inserted before every paragraph. A double-click on the middle icon opens a Size dialog box for that purpose. Note that new paragraphs are created by inserting either a ruler or a PARA character. A plain LINE character (inserted using the *return* key) does not begin a new paragraph! Finally, the last icon can be used to force a page break just before the ruler when printing. Pages are not printed if they are empty, i.e., at least a space character must be on a page for it to be printed.

The tab stop and margin marking bar supports direct manipulation of its components. Two triangles at the left end are used to control the indentation of the first line of every paragraph (upper triangle) and the left margin (lower triangle). A triangle at the right end controls the right margin. Small up-pointing triangles show positions of tabulator stops; a line under a tab stop triangle indicates the adjustment mode of that tab stop. New tab stops can be set by clicking into empty areas of the marking bar; old tab stops or the right margin can be removed by dragging them out of the ruler. Single tab stops can be moved by dragging them; all tabs stops to the right of and including the dragged one can be moved simultaneously by holding down the *modifier* key while dragging.

Successive clicking on a tab icon changes it cyclically from the default right-aligned tab to a centered tab to a left-aligned tab back to a right-aligned tab. Successive modifier-clicking on a tab icon toggles it from a normal tab to a bar tab and back again. Bar tabs show the tab position by a vertical bar.

Note that when a user enters a TAB character in a paragraph where there are no tab stops defined to the right side of the insertion point, the tab is interpreted as a fixed-size space. In other words: it doesn't hurt to enter TABs when no tab stops are defined; the text system defaults to a reasonable behavior in this case.

The text system uses an invisible default ruler where no specific ruler has been selected. To change the default ruler of a document, select a ruler which has the desired attributes, and then executed *Text->Make Default Ruler*. This allows (even for empty text documents) to set up template texts ("stationeries" on Mac OS) with the right attributes. For example, suitable tab stops could be set up in such a template text.

Double-clicking anywhere in the passive area, or between icons in the icon bar, opens an auxiliary dialog box that can be used to set ruler options that are not normally dealt with manually. Currently, two such options are controlled by this dialog, both of which affect the way text is broken across pages. The first option, *avoid page breaks inside*, asks the text setting mechanism to attempt to avoid a page break in the text between this and the next ruler (or the end of the text). If the text is longer than one page, a page break will occur anyway. The second option, *keep together with next*, asks for avoidance of page breaks between the text following this ruler and the text following the next ruler. (This option has no effect if this ruler is the last one in the text.) Again, a page break will occur if the resulting text block would exceed the length of one page.

<a id="Texts as"></a>**12 Texts as information environments**

The whole on-line documentation of BlackBox can be accessed by starting from the *Help* dialog box, following [<u>hyperlinks such as this one</u>](../../Std/Docu/Links.odc.md). The *Help* can be reached over the *Help* menu.

Mac OS:

In System 7, the *Help* menu is the second menubar icon from the right, i.e., the icon with a question mark in it.

If you click on a blue underlined text stretch with the mouse, a command is executed which opens another document, in this case the documentation of the standard link views. Hyperlinks allow the construction of arbitrary webs of information. Such a web can be customized for one's own use, in effect creating one's own *information environment*. To help structure a single text document, text stretches can be folded together in a hierarchical fashion, like here:

    This is a collapsed fold

If you click on one of the arrow symbols, the text is expanded into:

This is an expanded fold. Folded texts are delimited by [<u>fold views</u>](../../Std/Docu/Folds.odc.md), which are represented as arrows.

<a id="Editor and"></a>**13 Editor and browser modes**

Texts can be used in two different modes. Normally, a text is used in *edit mode*, i.e., it can be freely edited. Texts are saved in edit mode, and opened in document windows.

For hypertext documentation, a text can be opened in *browser mode* instead, in an auxiliary window. Such a text cannot be modified. However, its contents may be selected. The selection can be used to invoke commands on it, e.g., a find (but not a replace) command. Also, a selection may be copied into another document through drag & drop.

Documentation texts need not be saved in browser mode. They are saved in edit mode, and thus can be opened, edited, and saved again via the normal *File* menu commands. Hypertext documentation is opened via the *StdCmds.OpenBrowser* command, which opens a text document into an auxiliary window and forces it into browser mode. For example, the following commands show the difference between the two modes by opening the same (edit mode) text in the two possible ways:

 "StdCmds.OpenBrowser('Docu/Tut-A', 'A Brief History of BlackBox')"

 "StdCmds.OpenDoc('Docu/Tut-A')"

The latter command corresponds to the *File->Open* command and allows editing.

The *OpenBrowser* command accepts a *portable path name* as input. A portable path name is a string which denotes a file in a machine-independent way. It uses the "/" character as separator, i.e., like in Unix or the World-Wide Web.

The command is usually used in link views.

See also [<u>StdLinks</u>](../../Std/Docu/Links.odc.md) and [<u>StdCmds</u>](../../Std/Docu/Cmds.odc.md).

<a id="Summary of"></a>**14 Summary of keyboard shortcuts**

Besides the keyboard equivalents defined for the various menu commands, the following key or key combination can be used:

Function    Windows        Mac OS

OK / default button        return or enter

cancel button        esc

deselect        esc

activate object    modifier + enter

deactivate object    shift + esc

show properties    alt + enter

show context menu    shift + F10

delete (right)    delete    **    **delete or forward delete*

delete (left)    backspace        backspace or delete*

new line with auto indentation        return or enter

insert non-breaking space        modifier + space

insert hyphen        modifier + minus

insert soft hyphen    modifier + shift + minus        command + minus

insert non-breaking hyphen    alt + shift + minus        modifier + shift + minus

caret one character left        left arrow

caret one character right        right arrow

caret one word left        modifier + left arrow

caret one word right        modifier + right arrow

caret one screen left    modifier + page up

caret one screen right    modifier + page down

caret one line up        up arrow

caret one line down        down arrow

caret one paragraph up        modifier + up arrow

caret one paragraph down        modifier + down arrow

caret one screen up        page up

caret one screen down        page down

caret to beginning of line    home

caret to end of line    end

caret to beginning of document    modifier + home        home

caret to end of document    modifier + end        end

* Depending on the keyboard

Windows:

For compatibility with older versions of Windows, some obsolete keyboard shortcuts are still supported: *alt + backspace* for undo, *shift + delete* for cut, *modifier + insert* for copy, and *shift + insert* for paste.

<a id="New User"></a>**15 Windows 95 user interface guidelines (Windows only)**

BlackBox largely adopts the Windows interface guidelines defined in 1994 for Windows 95. Besides drag & drop, the guidelines also deal with the usage of the second mouse button. The secondary (usually the right) mouse button can be used like the primary button with the following differences: if drag & drop is invoked using the secondary button, a popup menu appears at the end. Such a menu (also called a *context menu*) contains the *Edit* menu entries *Cut*, *Copy*, *Paste*, and *Paste Object* plus additional commands depending on the actual selection.

In addition to the features defined by the interface guidelines, BlackBox supports two shortcuts for experienced users. Changing from move to copy during drag & drop can be done without touching the keyboard by clicking the second mouse button, while still holding down the first one. If a three-button mouse is connected (and supported by the installed driver), the middle button can be used for drag & pick.

<a id="Text Preferences"></a>**16 Text preferences (Windows only)**

To tune the text system for individual needs, some global parameters can be configured. The parameters need only be set once because they are stored in the Windows registry and are loaded automatically upon program start. The parameters are set up by the *Edit->Preferences...* command, which shows a preferences dialog box. The dialog box contains controls for changing the default font, the font metric used for text display, and the way windows are restored when they are scrolled by mouse dragging in a scroll bar. If *Use TrueType Metric * is set, the exact metric is used for the placement of individual characters. This gives better results on printers but usually leads to a hard-to-read screen display. If *Visual Scrolling* is enabled, the contents of a window is continuously updated during dragging of the the handle in the scroll bar. Otherwise a single update is performed, when the mouse button is released.

For more information on the *Text* subsystem's programming interface, consult the on-line documentation of the modules *TextModels*, *TextMappers*, *TextRulers*, *TextSetters*, *TextViews*, *TextControllers*, and *TextCmds*. Examples are given in the *Obx* subsystem, in particular the examples *ObxHello0*, *ObxHello1*, *ObxOpen0*, *ObxOpen1*, *ObxCaps*, *ObxDb*, *ObxTabs*, *ObxMMerge*, *ObxParCmd*, *ObxLinks*, and *ObxAscii*. A tutorial on the text subsystem is given in [<u>Chapter 5</u>](../../Docu/Tut-5.odc.md) of the accompaying book on component software and the BlackBox Component Framework.

