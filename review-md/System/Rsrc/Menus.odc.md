**MENU **"File"

    "&New"    "N"    "StdCmds.New"    ""

    "&Open..."    "O"    "HostCmds.Open"    ""

    "&Open Stationery..."    "*O"    "HostCmds.OpenCopyOf"    ""

    "&Save"    "S"    "HostCmds.Save"    "HostCmds.SaveGuard"

    "Save &As..."    "*S"    "HostCmds.SaveAs"    "StdCmds.WindowGuard"

    "Save Copy As..."    ""    "HostCmds.SaveCopyAs"    "StdCmds.WindowGuard"

    "&Close"    "W"    "HostCmds.Close"    "StdCmds.WindowGuard"

    **SEPARATOR**

    "Page Se&tup..."    ""    "HostDialog.InitPageSetup; StdCmds.OpenToolDialog('HostDialog.setup', 'Page Setup')"

                "StdCmds.WindowGuard"

    "&Print..."    "P"    "HostCmds.Print"    "HostCmds.PrintGuard"

    **SEPARATOR**

    "E&xit"    ""    "HostMenus.Exit"    ""

**END**

**MENU **"Edit"

    "&Undo"    "Z"    "StdCmds.Undo"    "StdCmds.UndoGuard"

    "R&edo"    "Y"    "StdCmds.Redo"    "StdCmds.RedoGuard"

    **SEPARATOR**

    "Cu&t"    "X"    "HostCmds.Cut"    "HostCmds.CutGuard"

    "&Copy"    "C"    "HostCmds.Copy"    "HostCmds.CopyGuard"

    "&Paste"    "V"    "HostCmds.Paste"    "HostCmds.PasteGuard"

    "&Delete    Delete"    ""    "StdCmds.Clear"    "HostCmds.CutGuard"

    "&Copy Properties"    "*C"    "StdCmds.CopyProp"    "StdCmds.SelectionGuard"

    "&Paste Properties"    "*V"    "StdCmds.PasteProp"    "StdCmds.SelectionGuard"

    **SEPARATOR**

    "Paste O&bject"    ""    "HostCmds.PasteObject"    "HostCmds.PasteObjectGuard"

    "Paste &Special..."    ""    "OleClient.PasteSpecial"    "HostCmds.PasteObjectGuard"

    "Paste to &Window"    ""    "HostCmds.PasteToWindow"    "HostCmds.PasteToWindowGuard"

    **SEPARATOR**

    "&Insert Object..."    ""    "OleClient.InsertObject"    "StdCmds.PasteViewGuard"

    "Object P&roperties...    Alt+Enter"    ""    "StdCmds.ShowProp"    "StdCmds.ShowPropGuard"

    "&Object"    ""    "HostMenus.ObjectMenu"    "HostMenus.ObjectMenuGuard"

    **SEPARATOR**

    "Select Docu&ment"    " "    "StdCmds.SelectDocument"    "StdCmds.WindowGuard"

    "Select &All"    "A"    "StdCmds.SelectAll"    "StdCmds.SelectAllGuard"

    "Select &Next Object"    "F6"    "StdCmds.SelectNextView"    "StdCmds.ContainerGuard"

    **SEPARATOR**

    "Pre&ferences..."    ""    "HostDialog.InitPrefDialog; StdCmds.OpenToolDialog('Host/Rsrc/Prefs', 'Preferences')"    ""

**END**

**MENU **"Attributes"

    "&Regular"    ""    "StdCmds.Plain"    "StdCmds.PlainGuard"

    **SEPARATOR**

    "&Bold"    "B"    "StdCmds.Bold"    "StdCmds.BoldGuard"

    "&Italic"    "I"    "StdCmds.Italic"    "StdCmds.ItalicGuard"

    "&Underline"    "U"    "StdCmds.Underline"    "StdCmds.UnderlineGuard"

    **SEPARATOR**

    " &8 Point"    ""    "StdCmds.Size(8)"    "StdCmds.SizeGuard(8)"

    " &9 Point"    ""    "StdCmds.Size(9)"    "StdCmds.SizeGuard(9)"

    "&10 Point"    ""    "StdCmds.Size(10)"    "StdCmds.SizeGuard(10)"

    "1&2 Point"    ""    "StdCmds.Size(12)"    "StdCmds.SizeGuard(12)"

    "1&6 Point"    ""    "StdCmds.Size(16)"    "StdCmds.SizeGuard(16)"

    "2&0 Point"    ""    "StdCmds.Size(20)"    "StdCmds.SizeGuard(20)"

    "2&4 Point"    ""    "StdCmds.Size(24)"    "StdCmds.SizeGuard(24)"

    "&Size..."    ""    "StdCmds.InitSizeDialog; StdCmds.OpenToolDialog('Std/Rsrc/Cmds', 'Size')"

                "StdCmds.SizeGuard(-1)"

    **SEPARATOR**

    "&Default Color"    ""    "StdCmds.Color(1000000H)"    "StdCmds.ColorGuard(1000000H)"

    "Blac&k"    ""    "StdCmds.Color(0000000H)"    "StdCmds.ColorGuard(0000000H)"

    "R&ed"    ""    "StdCmds.Color(00000FFH)"    "StdCmds.ColorGuard(00000FFH)"

    "&Green"    ""    "StdCmds.Color(000FF00H)"    "StdCmds.ColorGuard(000FF00H)"

    "B&lue"    ""    "StdCmds.Color(0FF0000H)"    "StdCmds.ColorGuard(0FF0000H)"

    "&Color..."    ""    "HostDialog.ColorDialog"    "StdCmds.ColorGuard(-1)"

    **SEPARATOR**

    "Default F&ont"    ""    "StdCmds.DefaultFont"    "StdCmds.DefaultFontGuard"

    "&Font..."    ""    "HostDialog.FontDialog"    "StdCmds.TypefaceGuard"

    "&Typeface..."    ""    "HostDialog.TypefaceDialog"    "StdCmds.TypefaceGuard"

**END**

[**<u>INCLUDE</u>**](StdMenuTool.ThisMenu) "Dev"

[**<u>INCLUDE</u>**](StdMenuTool.ThisMenu) "Text"

[**<u>INCLUDE</u>**](StdMenuTool.ThisMenu) "Form"

[**<u>INCLUDE</u>**](StdMenuTool.ThisMenu) "Sql"

[**<u>INCLUDE</u>**](StdMenuTool.ThisMenu) "Obx"

[**<u>INCLUDE</u>**](StdMenuTool.ThisMenu) "*"

**MENU **"&Window"

    "&New Window"    "F2"    "StdCmds.NewWindow"    "StdCmds.ModelViewGuard"

    **SEPARATOR**

    "&Cascade"    ""    "HostMenus.Cascade"    "StdCmds.WindowGuard"

    "Tile &Horizontal"    ""    "HostMenus.TileHorizontal"    "StdCmds.WindowGuard"

    "&Tile Vertical"    ""    "HostMenus.TileVertical"    "StdCmds.WindowGuard"

    "&Arrange Icons"    ""    "HostMenus.ArrangeIcons"    "StdCmds.WindowGuard"

    **SEPARATOR**

    "*"    ""    "HostMenus.WindowList"    ""

**END**

**MENU **"Help"

    "Contents"    "F1"    "StdCmds.OpenAuxDialog('Docu/Help', 'Help Contents')"    ""

    "Examples"    ""    "StdCmds.OpenBrowser('Obx/Docu/Sys-Map', 'Examples')"    ""

    **SEPARATOR**

    "About BlackBox"    ""    "StdCmds.OpenToolDialog('System/Rsrc/About', 'About BlackBox')"    ""

**END**

**MENU **"*"

    "Cu&t"    ""    "HostCmds.Cut"    "HostCmds.CutGuard"

    "&Copy"    ""    "HostCmds.Copy"    "HostCmds.CopyGuard"

    "&Paste"    ""    "HostCmds.Paste"    "HostCmds.PasteGuard"

    "&Delete"    ""    "StdCmds.Clear"    "HostCmds.CutGuard"

    **SEPARATOR**

    "P&roperties..."    ""    "StdCmds.ShowProp"    "StdCmds.ShowPropGuard"

    "&Object"    ""    "HostMenus.ObjectMenu"    "HostMenus.ObjectMenuGuard"

**END**

