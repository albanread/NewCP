**Converters**

DEFINITION Converters;

    IMPORT Stores, Files, Dialog;

    CONST importAll = 0;

    TYPE

        Importer = PROCEDURE (f: Files.File; OUT s: Stores.Store);

        Exporter = PROCEDURE (s: Stores.Store; f: Files.File);

        Converter = POINTER TO RECORD

            next-: Converter;

            imp-, exp-: Dialog.String;

            storeType-: Stores.TypeName;

            fileType-: Files.Type;

            opts-: SET

        END;

    VAR list-: Converter;

    PROCEDURE Register (imp, exp: Dialog.String; storeType: Stores.TypeName; fileType: Files.Type;

                                        opts: SET);

    PROCEDURE Import (loc: Files.Locator; name: Files.Name; VAR conv: Converter; OUT s: Stores.Store);

    PROCEDURE Export (loc: Files.Locator; name: Files.Name; conv: Converter; s: Stores.Store);

END Converters.

Module *Converters* allows the definition and registration of file converters. A file converter is an *importer* which translates a file into a store (usually some view type), or an *exporter* which translates a store into a file, or it is both an importer and an exporter simultaneously. For a given pair *(file type, store type)* there may be several file converters registered, e.g. a text view may be translated into an ASCII file with or without carriage returns at the end of lines.

Example: [<u>ObxConv  docu</u>](../../Obx/Docu/Conv.odc.md)

CONST **importAll**

Set element for the *opts* set of procedure *Register*. It indicates that the importer is able to import all file types (e.g., an importer that displays the file contents as a hex dump).

TYPE **Importer**

This procedure type is the signature of an importer command. An importer translates a given file *f* into a store *s*.

Pre

f # NIL    20

f has correct type    22

Post

s # NIL

TYPE **Exporter**

This procedure type is the signature of an exporter command. An exporter translates a given store *s* into the contents of a file *f*. File *f* is already set up as an empty new (i.e., writable) file.

Pre

s # NIL    20

f # NIL    21

f.Length() = 0    22

s has correct type    23

Post

f.Length() >= 0

TYPE **Converter**

A converter object represents a file converter. It consists of an import and an export command, one of which may be empty. A converter converts between a file and a store.

**next**-: Converter

Next converter in the list. Converters are sorted by their registration time: later registration means further back in the list.

**imp**-, **exp**-: Dialog.String    imp # "" OR exp # ""

Strings for the import/export commands, e.g.,

"HostTextConv.ImportText" or

"HostTextConv.ExportText".

**storeType**-: Stores.TypeName    exp # "" -> storeType # ""

Store type of the converter, e.g., "TextViews.TextView".

**fileType**-: Files.Type    fileType # ""

File type of the converter, e.g., "TXT".

**opts**-: SET

Set of options, e.g., {} or {importAll}.

VAR **list**-: Converter

List of registered converters. Converters are sorted by their registration time: later registration means further back in the list. The first element of the list, i.e., *list*, is always the document converter, i.e., the converter used for standard BlackBox document files.

PROCEDURE **Register** (imp, exp: Dialog.String; storeType: Stores.TypeName;

                                        fileType: Files.Type; opts: SET)

Register an importer which translates a file of type *fileType* into a store of type *storeType* (e.g., "TextViews.View"), an exporter which translates a store of type *storeType* into a file of type *fileType*, or both.

*imp* is the name of an importer command, which must have the signature of *Importer*.

*exp* is the name of an exporter command, which must have the signature of *Exporter*.

*opts* allows to express a set of options; currently only the element *importAll* is defined. Normally, *opts* is empty.

*Register* does not yet load the modules which contain the import/export commands. They are loaded only when needed.

The standard document converter is already installed by the BlackBox core. Other converters may be installed in the *Config* module, e.g., converters for ASCII files, Unicode files, or picture files. *Config* is executed upon startup of BlackBox to allow the establishment of custom configurations, such as the set of available converters.

As a result of *Register*, a new converter is appended to *list*, with fields corresponding to the parameters passed.

For each registered importer or exporter, there optionally may be a corresponding string mapping; to make the display of a list of importers/exporters more user-friendly. For example, the importer "HostTextConv.ImportText" could be mapped to the more telling name "Ascii" (in the standard file open dialog). The mapping is done in the *Strings* file in the *Rsrc* directory of the importer's subsystem, e.g., there may be the following lines in the *Host/Rsrc/Strings* text:

    HostTextConv.ImportText    Ascii

    HostTextConv.ExportText    Ascii

Pre

imp # "" OR exp # ""    20

fileType # ""    21

PROCEDURE **Import** (loc: Files.Locator; name: Files.Name; VAR conv: Converter; OUT s: Stores.Store)

Converts the contents of the file specified by *(loc, name)* into store *s*, using converter *conv*. Internally it calls the converter's import command. If *conv = NIL*, the first suitable converter in *list* is used and returned in the VAR parameter.

Pre

loc # NIL    20

name # ""    21

conv = NIL OR conv.imp # ""    22

File type of (loc, name) = converter's file type    23

PROCEDURE **Export** (loc: Files.Locator; name: Files.Name; conv: Converter; s: Stores.Store)

Convert store *s* to a new file *(loc, name)* using converter *conv*. Internally it calls the converter's export command. Success or failure is reported in the locator's *res* field.

Pre

s # NIL    20

~ s IS Stores.Alien    21

loc # NIL    22

name # ""    23

conv = NIL OR conv.exp # ""    24

TypeOf(s) = converter's store type    25

