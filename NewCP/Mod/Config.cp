MODULE Config;
(*
   First slice of the BlackBox `Config` port.

   BB's Config.Setup is a thin script of 19 `Converters.Register`
   calls that wires up the file-format handlers known to a stock
   BlackBox install (text / RTF / Unicode / hex / ETH / OLE …),
   followed by `Dialog.Call("StdLog.Open", "", res)` to put the
   system log on screen.

   This slice ships a slim `Setup` that registers the four
   converters we'll actually exercise (Documents.ImportDocument
   for `.odc`, HostTextConv.ImportText/ExportText for `.txt`)
   and calls `StdLog.Open` directly rather than via the Meta-
   reflection bounce that Dialog.Call would normally use (our
   Meta.LookupPath is still a surface stub).  Once Meta's
   reflection wires through, the Dialog.Call path lights up
   automatically.

   Deferred: the longer register list (RTF / Unicode / ETH /
   OLE / bitmap converters) — they map to modules
   (HostTextConv, HostBitmaps, StdETHConv, OleData) we haven't
   ported yet.  Restored once those land.
*)

    IMPORT Converters, StdLog;


    (** Called by `Init.Init` (currently directly, BB-faithful
        would go via Dialog.Call).  Idempotent — calling twice
        just re-registers identical entries. *)
    PROCEDURE Setup*;
    BEGIN
        (* Document handler: the .odc binary format. *)
        Converters.Register("Documents.ImportDocument",
                            "Documents.ExportDocument",
                            "",
                            "odc",
                            {});

        (* Plain text — HostTextConv hasn't ported yet, but
           registering the name now means once it lands, the
           dispatch works without a Config rebuild. *)
        Converters.Register("HostTextConv.ImportText",
                            "HostTextConv.ExportText",
                            "TextViews.View",
                            "txt",
                            {Converters.importAll});

        StdLog.Open
    END Setup;

END Config.
