**Overview by Example: ObxMailMerge**

When a business wants to communicate something to all its customers, e.g. the announcement of a new product, it can send out *form letters* to all its existing customers. Each such letter has the same contents, except for a few differences like the name and address of the respective customer. *Mail merge* is the process of creating these form letters out of a letter template and a customer database.

This example shows how the BlackBox Component Builder's built-in text component can be used to implement a simple mail merge extension.

Figure 1. Mail Merge Process

The following menu command is installed in menu *Obx*, to simplify trying out this example:

    ...

    "Merge..."    ""    "ObxMMerge.Merge"    "TextCmds.FocusGuard"

    ...

Before you execute *Obx->Merge...*, you need to open a mail merge template. For example, the following [<u>template document</u>](../Samples/mmTmpl.odc.md)

(Obx/Samples/MMTmpl) and then execute the *Obx->Merge...* command. A dialog box will ask you for the address database: open the mail merge [<u>data document</u>](../Samples/mmData.odc.md) (Obx/Samples/MMData).

As a result, a new text is created which contains a sequence of form letters. When you execute the *Show Marks* command in the *Text* menu, you'll note that the individual letters are separated by page-breaking rulers (the right-most icon in a ruler).

Now let's take a closer look at how the program is implemented. There is one command called *Merge*, which fetches the template text (the focus), searches for place holders in this text (the fields of the template), then lets the user open the database text, determines for each template field which column of the database text corresponds to the field, creates a new output text, adds an instance of the letter template for every row of the database text, and finally opens the text in a window.

MODULE ObxMailMerge;

    PROCEDURE TmplFields (t: TextModels.Model): Field;

    PROCEDURE ThisDatabase (): TextModels.Model;

    PROCEDURE MergeFields (f: Field; t: TextModels.Model);

    PROCEDURE ReadTuple (f: Field; r: TextModels.Reader);

    PROCEDURE AppendInstance (f: Field; data, tmpl, out: TextModels.Model);

    PROCEDURE **Merge***;

END ObxMailMerge.

Listing 2. Outline of the ObxMMerge Program

There are five auxiliary procedures which are called by *Merge*:

*TmplFields* analyzes the template text, and returns a list of fields for this text. Each field describes a place holder with its name and its position in the text. Place holders are specified as names between "<" and ">" characters, e.g., <Name>.

*ThisDatabase* asks the user for a database document, and returns the text contained in this document.

*MergeFields* determines for every template field the corresponding database column. To make this possible, the first row of the database text must contain the so-called *meta data* of the database. For mail merge applications, this is simply the symbolic name of every column, e.g., *Name* or *City*. This name must be identical to the name used in the template.

Each row is terminated by a carriage return (0DX), and the fields of a row (i.e., the columns) are separated by tabs (09X).

*ReadTuple* reads one row of the database text, and assigns the string occupied by one database field to every corresponding template field.

*AppendInstance* appends a copy of the template text to the end of the output text, and then replaces all the place holders by the contents of their respective database fields. These replacements are done from the end of the appended text towards the beginning, so that from/to indices are not invalidated by replacements. This explains why the field list is built up in reverse order, last field first.

Note that each replacement gets the text attributes of the first replaced character, i.e., if the place holder "<**Name**>" in bold face is replaced by the string "Joe", the resulting replacement will be '**Joe**".

*ObxMMerge* doesn't need to insert page-breaking rulers; instead, the template text contains such a ruler.

[<u>ObxMMerge  sources</u>](../Mod/MMerge.odc.md)

