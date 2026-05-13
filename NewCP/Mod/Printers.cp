MODULE Printers;
(*
   Direct port of BlackBox `System/Mod/Printers.odc` — small
   enough (~108 lines) to ship the whole module in one slice.

   `Printers` is the abstract API the framework uses to send
   a View tree to a printer.  Every concrete host (Windows /
   Mac / X11) installs a `Directory` subclass via `SetDir`
   at startup; `Directory.Default` / `Directory.Current`
   then hand out a `Printer` whose abstract `OpenJob` /
   `OpenPage` / `ClosePage` / `CloseJob` walk the print job
   lifecycle.  Concrete `Printer` subclasses live in the
   host layer.

   This slice has no host concrete subclass — the abstract
   surface alone is enough for `TextSetters` and friends
   to import a `Printer` parameter type without dragging in
   any UI plumbing.
*)

    IMPORT Ports;

    TYPE
        (** Abstract printer.  Carries the paper rectangle in
            port coordinates plus a non-NIL `Ports.Port` to
            paint into. *)
        PrinterDesc* = ABSTRACT RECORD
            l, t, r, b: INTEGER;       (** paper rect *)
            res*:       INTEGER;
            port:       Ports.Port
        END;
        Printer* = POINTER TO PrinterDesc;

        (** Abstract directory — the per-host factory that
            hands out `Printer` instances. *)
        DirectoryDesc* = ABSTRACT RECORD END;
        Directory* = POINTER TO DirectoryDesc;


    VAR
        (** The active directory and the default directory.
            Equal at startup; the active one may be swapped
            by host commands.  Both are read-only-exported
            so framework callers can probe availability. *)
        dir-, stdDir-: Directory;


    (* -- Printer abstract method declarations -------------------------- *)

    PROCEDURE (p: Printer) OpenJob*  (VAR copies: INTEGER; IN name: ARRAY OF CHAR), NEW, ABSTRACT;
    PROCEDURE (p: Printer) CloseJob* (), NEW, ABSTRACT;
    PROCEDURE (p: Printer) OpenPage* (), NEW, ABSTRACT;
    PROCEDURE (p: Printer) ClosePage* (), NEW, ABSTRACT;

    (** Optional landscape switch — EMPTY default so printers
        that only do portrait don't need to override. *)
    PROCEDURE (p: Printer) SetOrientation* (landscape: BOOLEAN), NEW, EMPTY;


    (* -- Printer concrete helpers -------------------------------------- *)

    (** Bind the Port the printer paints into.  Idempotent if
        the same port re-binds. *)
    PROCEDURE (p: Printer) InitPort* (port: Ports.Port), NEW;
    BEGIN
        ASSERT((p.port = NIL) OR (p.port = port), 20);
        p.port := port
    END InitPort;

    PROCEDURE (p: Printer) ThisPort* (): Ports.Port, NEW;
    BEGIN
        RETURN p.port
    END ThisPort;

    PROCEDURE (p: Printer) GetRect* (OUT l, t, r, b: INTEGER), NEW;
    BEGIN
        l := p.l;  t := p.t;  r := p.r;  b := p.b
    END GetRect;

    (** Initialise the paper rect.  Concrete `OpenJob`
        bodies call this once they've negotiated paper size
        with the host driver. *)
    PROCEDURE (p: Printer) InitPrinter* (l, t, r, b: INTEGER), NEW;
    BEGIN
        ASSERT(l <= r, 20);
        ASSERT(t <= b, 21);
        p.l := l;  p.t := t;  p.r := r;  p.b := b;
        p.res := 0
    END InitPrinter;


    (* -- Directory abstract methods ------------------------------------ *)

    PROCEDURE (d: Directory) Default*   (): Printer, NEW, ABSTRACT;
    PROCEDURE (d: Directory) Current*   (): Printer, NEW, ABSTRACT;
    PROCEDURE (d: Directory) Available* (): BOOLEAN, NEW, ABSTRACT;


    (** Install the active directory.  First call also
        becomes the immutable `stdDir`. *)
    PROCEDURE SetDir* (d: Directory);
    BEGIN
        ASSERT(d # NIL, 20);
        dir := d;
        IF stdDir = NIL THEN stdDir := d END
    END SetDir;


END Printers.
