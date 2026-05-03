**ObxWordEdit**

This example shows some possible usage of the Automation interface of Microsoft Word 9.0. See also under:

mk:@MSITStore:I:\MSDN\off2000.chm::/html/Off2000LangRef.htm

in the MSDN Library.

For more information about automation controllers in BlackBox see the [<u>Ctl Docu</u>](../../Ctl/Docu/Dev-Man.odc.md) and the<u> </u>[<u>CtlWord9 Docu</u>](../../Ctl/Docu/Word9.odc.md).

The module consists of a set of commands. Most of them require that the module is connected to a Word application. Before using them, ObxWordEdit.Connect or ObxWordEdit.Start must be called. The commands which act on a document also require that a document is open. Always the active document is used.

[<u>ObxWordEdit  sources</u>](../Mod/WordEdit.odc.md)

**Connecting / Disconnecting**

ObxWordEdit.Connect

Connects to a Word application. If Word is not running, it is started.

ObxWordEdit.Disconnect

Disconnects from Word. If nobody else is using the application, it is terminated.

**Starting / Quitting**

ObxWordEdit.Start

Starts a new Word application.

ObxWordEdit.Quit

Quits the Word.

ObxWordEdit.Restart

Restarts Word.

**File**

ObxWordEdit.NewDoc

Creates a new document using CtlWord.NewDocument() and makes it visible.

The document is created in the oldest running Word. If there is no Word running, it is started.

ObxWordEdit.CreateDoc

Creates a new, visible document using CtlWord.Application.Documents().Add.

It is only visible, if the application is visible.

ObxWordEdit.CreateInvisibleDoc

Creates a new, invisible document. It is also invisible, if the application is visible.

ObxWordEdit.OpenDocTest

Opens a test file.

ObxWordEdit.CloseDoc

Closes the active document without saving.

ObxWordEdit.SaveAndCloseDoc

Saves the active document and closes it.

ObxWordEdit.SaveDoc

Saves the active document.

ObxWordEdit.Print

Prints the active document.

**Application**

ObxWordEdit.DispDocs

Displays the full names of all the open documents.

ObxWordEdit.DispFontNames

Displays the names of the available fonts in Word.

ObxWordEdit.DispBBFontNames

Displays the names of the available fonts in BlackBox. There are more than in Word.

ObxWordEdit.DispLanguages

Displays the languages available in Word.

ObxWordEdit.DispLanguagesAndDictionaries

Displays the languages available in Word and whether they have a dictionary.

Attention: ActiveSpellingDictionary traps if there is no dictionary available...

ObxWordEdit.UseSmartCutPaste

Sets the option SmartCutPaste.

**Visibility**

ObxWordEdit.MakeWordVisible

Makes all documents visible, also the ones that were created invisible.

ObxWordEdit.MakeWordInvisible

Makes all documents invisible.

ObxWordEdit.MakeWinVisible

Makes the first window of the active document visible.

ObxWordEdit.MakeWinInvisible

Makes the first window of the active document invisible.

ObxWordEdit.IsVisible

Displays whether the first window of the active document is visible.

**Document**

ObxWordEdit.Undo

Undoes the last action. Actions, such a typing characters, can be merged to one action by Word.

ObxWordEdit.Redo

Redoes the last undone action.

ObxWordEdit.Protect

Protects the active document.

ObxWordEdit.Unprotect

Unprotects the active document.

**Accessing the Content of a Document**

ObxWordEdit.DispContent

Displays the content of the active document.

ObxWordEdit.DispParagraphs

Displays the paragraphs of the active document.

ObxWordEdit.DispListParagraphs

Displays the ListParagraphs of the active document.

ObxWordEdit.DispLists

Displays the Lists of the active document.

ObxWordEdit.DispWords

Displays the Words of the active document, using the CtlWord.Document.Words method.

ObxWordEdit.DispWords2

Displays the Words of the active document, using the CtlWord.Range.Next method.

ObxWordEdit.DispWords3

Displays the Words of the active document, using the CtlWord.Range.MoveEnd method.

ObxWordEdit.DispCharacters

Displays the Characters of the active document.

ObxWordEdit.DispSentences

Displays the Sentences of the active document.

ObxWordEdit.DispStoryRanges

Displays the StoryRanges of the active document.

ObxWordEdit.DispRuns

Should write the runs of the active document, but it does not work! How can we get the runs?

**Editing Text**

ObxWordEdit.AppendThisText

Appends "This Text" to the end of the active document. This means that it is insert in front of the last 0DX.

ObxWordEdit.OverwriteLastCharWithA

Overwrites the last character, which is always a 0DX, with "A". Word inserts a new 0DX at the end.

ObxWordEdit.CopyText

Copies and inserts the first 10 chars at the beginning of the active document.

ObxWordEdit.CopyFormattedText

Copies and inserts the first 10 chars at the beginning of the active document, formatted.

ObxWordEdit.DeleteText

 Deletes the first 10 character of the active document.

**Font**

ObxWordEdit.Disp2ndParagraphFontName

Displays the name of the font of the second paragraph of the active document, if it is defined.

ObxWordEdit.Is1stParagraphBold

Displays whether the first paragraph of the active document is bold or not, if it is defined.

ObxWordEdit.Get1stParagraphFontSize

Displays the size of the font of the first paragraph of the active document, if it is defined.

ObxWordEdit.Disp1stParagraphFontSize

Displays the size of the font of the first paragraph of the active document, if it is defined.

**Performance**

ObxWordEdit.Performance

Shows how much slower it is to call a procedure in Word via the automation interface.

