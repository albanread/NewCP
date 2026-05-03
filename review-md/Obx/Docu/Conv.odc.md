**Overview by Example: ObxConv**

While most data files are BlackBox document files, it is sometimes necessary to deal with other file types as well: ASCII files, picture files, or files created by  legacy applications. Sometimes it is sufficient to import such data once and then keep it in BlackBox documents, sometimes it is necessary to export data into some foreign file format, and it may even be desirable to import data from such a file format when opening a file, and to export data back into the same file format upon saving the file. Of course, the latter only works in a satisfying way if the conversion is truly invertible, i.e., both import and export are loss-free.

    The example *ObxConv* shows a simple ASCII converter, which converts to and from standard BlackBox text views. The module consists of two commands, one is an importer and the other an exporter. Both commands have the same structure: in a loop over all characters of a text, each character is read (from a file or text) and then written (into a text or file).

    Converters are often platform-dependent. In our example this can be seen in the assertion which checks the correct file type. Under Windows, it should read

    ASSERT(f.type = "TXT", 22)    while under Mac OS it should read

    ASSERT(f.type = "TEXT", 22)

Furthermore, the characters would have to be converted from the host platform's native character set to (or from) the portable character set used by the BlackBox Component Builder. This is not done here since it would not contribute to demonstrating the converter mechanism per se.

    Each BlackBox Component Builder implementation provides its own set of standard converters which are registered and thus made available to the user when the system is started up. A programmer may install additional converters in procedure *Config.Setup*, by calling the *Converters.Register* procedure:

Windows:

Converters.Register("ObxConv.ImportText", "ObxConv.ExportText", "TextViews.View", "TXT", {})

Mac OS:

Converters.Register("ObxConv.ImportText", "ObxConv.ExportText", "TextViews.View", "TEXT", {})

*Config.Setup* is called at the end of the BlackBox Component Builder's bootstrap process, i.e., when the core has been loaded successfully.

For each registered converter, there optionally may be a corresponding string mapping; to make the display of a list of importers/exporters more user-friendly. For example, the importer "ObxConv.ImportText" could be mapped to the more telling name "Obx Text" (in the standard file open dialog). The mapping is done in the *Strings* file in the *Rsrc* directory of the importer's subsystem, e.g. there is the following line in the *Obx/Rsrc/Strings* text:

    ObxConv.ImportText    Obx Text File

It is conceivable to implement compound converters: an HTML (Hypertext Markup Language) converter for example could use the standard ASCII converter of the BlackBox Component Builder (i.e., module *HostTextConv*) to perform a conversion from the platform's ASCII format to an BlackBox text. The created text could then be parsed and converted into an equivalent text which included some kind of hypertext link view instead of the textually specified hyperlinks.

[<u>ObxConv  sources</u>](../Mod/Conv.odc.md)

