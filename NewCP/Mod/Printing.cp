MODULE Printing;
(*
   First slice of the BlackBox `Printing` port.

   BB's `Printing` is the page-layout state owned by `Document`
   — page-info, headers/footers, copies count, and a hook for the
   actual printer-driver invocation.  Documents declares fields
   of type `Printing.Par` / `Printing.Hook` and references
   `Printing.par` / `Printing.Current` from a small handful of
   call sites; the heavy printing machinery (banner rendering,
   page composition) is gated behind `Documents.Print` which is
   not on the welcome-page open path.

   This slice ships the surface — every type, constant, and
   public procedure heading Documents and Windows reference at
   the type level.  Bodies that don't fire on the welcome-page
   path return safe defaults; PrintView / PrintBanner / hook
   dispatch are stubbed pending the host print pipeline porting.

   Deferred:
     - PrintView / PrintBanner real bodies (need text-layout
       walker + page-break logic).
     - Hook chain dispatch (no host printing backend yet).
*)

    IMPORT Kernel, Fonts, Views;

    CONST
        maxNrOfSegments = 16;


    TYPE
        (** BB-faithful page descriptor.  Fields carry the page
            range (`first`/`from`/`to`), the "alternate-page-
            layout" flag, and the title used by header / footer
            substitution.  All exported because Documents passes
            this directly to user code. *)
        PageInfo* = RECORD
            first*, from*, to*: INTEGER;
            alternate*:         BOOLEAN;
            title*:             Views.Title
        END;

        (** BB-faithful banner (page header or footer).  Carries
            the font + a left- and right- justified text segment
            (with BB-style `&` substitutions like `&p` for page
            number — handled by PrintBanner once that's ported).
            `gap` is the inter-banner gap. *)
        Banner* = RECORD
            font*:         Fonts.Font;
            gap*:          INTEGER;
            left*, right*: ARRAY 128 OF CHAR
        END;

        (** BB-faithful print-job parameters — the value passed
            through `Documents.Print` and the print-dialog
            machinery.  LIMITED in BB so callers go through
            `NewPar` / `NewDefaultPar` rather than constructing
            one inline. *)
        Par* = POINTER TO LIMITED RECORD
            page*:           PageInfo;
            header*, footer*: Banner;
            copies-:         INTEGER
        END;

        (** Print-driver hook — subclassed by the host printing
            backend (none ported yet).  Our `Kernel.Hook` is the
            POINTER alias; `Kernel.HookDesc` is the ABSTRACT
            record we extend (BB conflates the two names with
            its POINTER-TO-ABSTRACT-RECORD shorthand). *)
        HookDesc* = ABSTRACT RECORD (Kernel.HookDesc) END;
        Hook*     = POINTER TO HookDesc;


    VAR
        (** The active print-job parameters.  Documents reads
            this through `con.param`; the welcome-page open path
            never sets it. *)
        par*: Par;

        printingHook: Hook;


    PROCEDURE (h: HookDesc) Print* (v: Views.View; par: Par), NEW, ABSTRACT;
    PROCEDURE (h: HookDesc) Current* (): INTEGER, NEW, ABSTRACT;


    (** Install the printing hook.  Symmetric with BB's signature;
        no-op until a host print backend ports. *)
    PROCEDURE SetHook* (p: Hook);
    BEGIN
        printingHook := p
    END SetHook;

    (** Allocate a new Par with the supplied page / header /
        footer / copies.  BB-faithful — defaults the missing font
        slots to the system default font. *)
    PROCEDURE NewPar* (IN page: PageInfo; IN header, footer: Banner; copies: INTEGER): Par;
        VAR p: Par;
    BEGIN
        NEW(p);
        p.page   := page;
        p.header := header;
        p.footer := footer;
        p.copies := copies;
        IF p.header.font = NIL THEN p.header.font := Fonts.dir.Default() END;
        IF p.footer.font = NIL THEN p.footer.font := Fonts.dir.Default() END;
        RETURN p
    END NewPar;

    (** Build the conventional default Par with the supplied
        document title — first/from/to = 0/0/9999, no alternate
        layout, empty header / footer. *)
    PROCEDURE NewDefaultPar* (title: Views.Title): Par;
        VAR pg: PageInfo; hd, ft: Banner;
    BEGIN
        pg.first := 0; pg.from := 0; pg.to := 9999;
        pg.alternate := FALSE; pg.title := title;
        hd.gap := 0; hd.left := ""; hd.right := "";
        ft.gap := 0; ft.left := ""; ft.right := "";
        RETURN NewPar(pg, hd, ft, 1)
    END NewDefaultPar;

    (** Drive the printing hook for `view`.  Deferred — no host
        printing backend yet; the call is a no-op so framework
        callers (`Documents.Print` ...) don't trap. *)
    PROCEDURE PrintView* (view: Views.View; p: Par);
    BEGIN
        IF printingHook # NIL THEN
            printingHook.Print(view, p)
        END
    END PrintView;

    (** Render a single banner (header or footer).  Deferred —
        the real body walks the `&p` / `&d` / ... substitutions
        and lays out segments at left / right of the page; needs
        the text-measurement helpers we haven't wired into Ports
        yet. *)
    PROCEDURE PrintBanner* (f: Views.Frame; IN p: PageInfo; IN b: Banner;
                            date, time, x0, x1, y: INTEGER);
    BEGIN
        (* no-op: deferred until banner layout + & substitution port *)
    END PrintBanner;

    (** Current page number under the active print job.  Deferred
        — driven by the hook once it's installed; returns 1
        otherwise so banner code that does `(first + Current()) MOD 2`
        produces a sensible value. *)
    PROCEDURE Current* (): INTEGER;
    BEGIN
        IF printingHook # NIL THEN
            RETURN printingHook.Current()
        ELSE
            RETURN 1
        END
    END Current;


BEGIN
    par := NIL;
    printingHook := NIL
END Printing.
