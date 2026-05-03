**MENU **"SQL"

    "Browser..."    ""    "StdCmds.OpenAuxDialog('Sql/Rsrc/Browser', 'Browser')"    ""

    "Execute"    ""    "SqlBrowser.ExecuteSel"    "TextCmds.SelectionGuard"

    **SEPARATOR**

    "Company..."    ""    "SqlObxDB.Open; StdCmds.OpenAuxDialog('Sql/Rsrc/Company', 'Company')"    ""

    "Ownership..."    ""    "SqlObxExt.Open; StdCmds.OpenAuxDialog('Sql/Rsrc/Owner', 'Ownership')"    ""

    **SEPARATOR**

    "Insert Anchor"    ""    "SqlControls.DepositAnchor; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Insert Table"    ""    "SqlControls.DepositTable; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Open Table"    ""    "SqlControls.DepositTable; StdCmds.Open"    ""

    **SEPARATOR**

    "Debug Options..."    ""    "StdCmds.OpenAuxDialog('Sql/Rsrc/Debug', 'Debug Options')"    ""

    **SEPARATOR**

    "Help"    ""    "StdCmds.OpenBrowser('Sql/Docu/Dev-Man', 'Sql Docu')"    ""

**END**

