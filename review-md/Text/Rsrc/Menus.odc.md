**MENU **"Text" ("TextViews.View")

    "&Find / Replace..."    "F"    "TextCmds.InitFindDialog; StdCmds.OpenToolDialog('Text/Rsrc/Cmds', 'Find / Replace')"    ""

    "Find &Again"    "F3"    "TextCmds.InitFindDialog; TextCmds.FindAgain('~I~B~E~R')"        "TextCmds.FindAgainGuard"

    "Find &Previous"    "*F3"    "TextCmds.InitFindDialog; TextCmds.FindAgain('~I~B~ER')"        "TextCmds.FindAgainGuard"

    "Find First"    "F4"    "TextCmds.InitFindDialog; TextCmds.FindFirst('~I~B~E~R')"        "TextCmds.FindAgainGuard"

    "Find Last"    "*F4"    "TextCmds.InitFindDialog; TextCmds.FindFirst('~I~B~ER')"        "TextCmds.FindAgainGuard"

    **SEPARATOR**

    "Shift &Left"    "F11"    "TextCmds.ShiftLeft"    "TextCmds.EditGuard"

    "Shift &Right"    "F12"    "TextCmds.ShiftRight"    "TextCmds.EditGuard"

    "Su&perscript"    ""    "TextCmds.Superscript"    "TextCmds.SelectionGuard"

    "Su&bscript"    ""    "TextCmds.Subscript"    "TextCmds.SelectionGuard"

    **SEPARATOR**

    "&Insert Paragraph"    "M"    "TextCmds.InsertParagraph"    "StdCmds.PasteCharGuard"

    "Insert R&uler"    "J"    "TextCmds.InsertRuler"    "StdCmds.PasteViewGuard"

    "Insert &Soft-Hyphen    Shift+Ctrl+Minus"    ""    "TextCmds.InsertSoftHyphen"    "StdCmds.PasteCharGuard"

    "Insert &Non-Brk Hyphen    Shift+Alt+Minus"    ""    "TextCmds.InsertNBHyphen"    "StdCmds.PasteCharGuard"

    "Insert N&on-Brk Space    Shift+Alt+Space"    ""    "TextCmds.InsertNBSpace"    "StdCmds.PasteCharGuard"

    "Insert &Digit Space    Shift+Ctrl+Space"    ""    "TextCmds.InsertDigitSpace"    "StdCmds.PasteCharGuard"

    "Toggle &Marks"    "H"    "TextCmds.ToggleMarks"    "TextCmds.ToggleMarksGuard"

    **SEPARATOR**

    "Make Default Attributes"    ""    "TextCmds.MakeDefaultAttributes"    "TextCmds.SelectionGuard"

    "Make Default Ruler"    ""    "TextCmds.MakeDefaultRuler"    "StdCmds.SingletonGuard"

**END**

**MENU **"*" ("TextViews.View")

    "Cu&t"    ""    "HostCmds.Cut"    "HostCmds.CutGuard"

    "&Copy"    ""    "HostCmds.Copy"    "HostCmds.CopyGuard"

    "&Paste"    ""    "HostCmds.Paste"    "HostCmds.PasteGuard"

    "&Delete"    ""    "StdCmds.Clear"    "HostCmds.CutGuard"

    **SEPARATOR**

    "&Source"    ""    "DevReferences.ShowSource"    "TextCmds.SelectionGuard"

    "&Interface"    ""    "DevBrowser.ShowInterface('')"    "TextCmds.SelectionGuard"

    "&Documentation"    ""    "DevReferences.ShowDocu"    "TextCmds.SelectionGuard"

    **SEPARATOR**

    "P&roperties..."    ""    "StdCmds.ShowProp"    "StdCmds.ShowPropGuard"

    "&Object"    ""    "HostMenus.ObjectMenu"    "HostMenus.ObjectMenuGuard"

**END**

