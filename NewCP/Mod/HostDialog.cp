MODULE HostDialog;
(*
   First slice of the BlackBox `HostDialog` port.

   BB's HostDialog is a large (~1900-line) module — it implements
   open / save / print / page-setup / color / font dialogs against
   the Win32 common-dialog API.  The only surface the welcome-page
   chain actually reaches for is the two status / message bars:
   `ShowParamMsg` (popup alert with `&0`/`&1`/`&2` substitutions)
   and `ShowParamStatus` (transient status-bar message).

   This slice ships only those two procedures, as no-op stubs.
   Every other HostDialog body (Open / Save / Print / Color /
   Font / Page Setup dialogs) is deferred until we need it.

   Deferred: ImpOk, GetIntSpec, GetExtSpec, Deposit, SetupNotify,
   SetupOk, InitPageSetup, PrintDialog, PrintSetup, CloseDialog,
   ColorDialog, FontDialog, TypefaceDialog, DefFont, DlgFont,
   PrefOk, InitPrefDialog, Start.
*)

    (** Modal message popup with parameter substitution.  BB's
        body looks up `str` in the resource catalog, substitutes
        `&0`/`&1`/`&2` with `p0`/`p1`/`p2`, then displays a
        MessageBox.  Stub for now — Dialog.ShowParamMsg already
        no-ops the upstream call. *)
    PROCEDURE ShowParamMsg* (IN str, p0, p1, p2: ARRAY OF CHAR);
    BEGIN
    END ShowParamMsg;

    (** Transient status-bar message.  Stub — same deferral. *)
    PROCEDURE ShowParamStatus* (IN str, p0, p1, p2: ARRAY OF CHAR);
    BEGIN
    END ShowParamStatus;

END HostDialog.
