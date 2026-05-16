MODULE Windows;
(*
   Abstract window surface — Pass 1.

   Declares the Window / Directory OOP hierarchy without any OS coupling.
   The concrete implementation lives in HostWindows.cp, which extends these
   types and registers a concrete Directory via SetDir.

   Deferred:
   - Controller / focus / keyboard routing.
   - Window flags (modal, floating, aux, …).
   - Sequencer / tick integration.
*)

    IMPORT Documents;

    TYPE
        WindowDesc* = ABSTRACT RECORD
            next-: Window
        END;
        Window* = POINTER TO WindowDesc;

        DirectoryDesc* = ABSTRACT RECORD END;
        Directory*     = POINTER TO DirectoryDesc;

    VAR
        dir-:    Directory;
        stdDir-: Directory;
        first-:  Window;


    (* -- Abstract methods on Window ---------------------------------------- *)

    PROCEDURE (w: Window) IsValid*   (): BOOLEAN, NEW, ABSTRACT;
    PROCEDURE (w: Window) ThisDoc*   (): Documents.Document, NEW, ABSTRACT;
    PROCEDURE (w: Window) SetTitle*  (IN title: ARRAY OF CHAR), NEW, ABSTRACT;
    PROCEDURE (w: Window) GetTitle*  (OUT title: ARRAY OF CHAR), NEW, ABSTRACT;
    PROCEDURE (w: Window) SetSize*   (width, height: INTEGER), NEW, ABSTRACT;
    PROCEDURE (w: Window) GetSize*   (OUT width, height: INTEGER), NEW, ABSTRACT;
    PROCEDURE (w: Window) Scroll*    (dx, dy: INTEGER), NEW, ABSTRACT;
    PROCEDURE (w: Window) Close*     (), NEW, ABSTRACT;


    (* -- Abstract method on Directory -------------------------------------- *)

    PROCEDURE (d: Directory) New* (doc: Documents.Document;
                                   IN title: ARRAY OF CHAR;
                                   w, h: INTEGER): Window, NEW, ABSTRACT;


    (* -- Module-level ------------------------------------------------------ *)

    PROCEDURE SetDir* (d: Directory);
    BEGIN
        ASSERT(d # NIL, 20);
        dir := d;
        IF stdDir = NIL THEN stdDir := d END
    END SetDir;

    PROCEDURE Open* (doc: Documents.Document;
                     IN title: ARRAY OF CHAR;
                     w, h: INTEGER): Window;
        VAR win: Window;
    BEGIN
        IF dir = NIL THEN RETURN NIL END;
        win := dir.New(doc, title, w, h);
        IF win # NIL THEN
            win.next := first;
            first    := win
        END;
        RETURN win
    END Open;


BEGIN
    dir    := NIL;
    stdDir := NIL;
    first  := NIL
END Windows.
