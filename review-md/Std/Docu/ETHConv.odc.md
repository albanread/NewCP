**StdETHConv**

DEFINITION StdETHConv;

    IMPORT Files, Stores, TextModels;

    PROCEDURE ImportETHDoc (f: Files.File; OUT s: Stores.Store);

    PROCEDURE ImportOberon (f: Files.File): TextModels.Model;

END StdETHConv.

Module *StdETHConv* provides an importer for ETH Oberon V4 text files.

PROCEDURE **ImportETHDoc** (f: Files.File; OUT s: Stores.Store)

Importer for Oberon V4 text files. Can be registered by module *Config* with the statement:

Windows:

    Converters.Register("StdETHConv.ImportETHDoc", "", "TextViews.View", ".ETH", {})

Mac OS:

    Converters.Register("StdETHConv.ImportETHDoc", "", "TextViews.View", ".Ob.", {})

PROCEDURE **ImportOberon** (f: Files.File): TextModels.Model

Directly converts an Oberon V4 text file into a BlackBox text, without using the converter mechanism of module *Converters*.

