MODULE TextRulersExtBase;
(*
   Workout for the `TextRulers` slice.

   Exercises:
   - extending the ABSTRACT `Directory` with concrete
     `NewStyle` and `New` factories that hand out leaf
     `Style` and `Ruler` instances;
   - extending `StyleDesc` and `RulerDesc` (ABSTRACT) with
     concrete leaves so the abstract chain is satisfied;
   - reading the inherited `attr` field on a Style and the
     `style` field on a Ruler — proves the Models.Model /
     Views.View / Containers.View base chains survive an
     extra cross-module hop through TextRulers;
   - reading inline tab-table state through `CopyTabs`;
   - constructing and inspecting a TextRulers.Prop.
*)

    IMPORT Stores, Models, Views, Containers, Properties, TextRulers;

    TYPE
        MyStyleDesc* = RECORD (TextRulers.StyleDesc) END;
        MyStyle* = POINTER TO MyStyleDesc;

        MyRulerDesc* = RECORD (TextRulers.RulerDesc) END;
        MyRuler* = POINTER TO MyRulerDesc;

        MyDirectoryDesc* = RECORD (TextRulers.DirectoryDesc) END;
        MyDirectory* = POINTER TO MyDirectoryDesc;


    (* Required ABSTRACT-method overrides.  Style extends
       Models.Model directly, no pure-ABSTRACTs there.
       Ruler extends Views.View whose only pure-ABSTRACT
       is Restore. *)
    PROCEDURE (r: MyRulerDesc) Restore*
        (f: Views.Frame; l, t, r0, b: INTEGER);
    BEGIN
    END Restore;


    (* Concrete Directory factories. *)

    PROCEDURE (d: MyDirectoryDesc) NewStyle* (attr: TextRulers.Attributes): TextRulers.Style;
        VAR s: MyStyle;
    BEGIN
        NEW(s);
        s.attr := attr;
        RETURN s
    END NewStyle;

    PROCEDURE (d: MyDirectoryDesc) New* (style: TextRulers.Style): TextRulers.Ruler;
        VAR r: MyRuler;
    BEGIN
        NEW(r);
        r.style := style;
        RETURN r
    END New;


    PROCEDURE Run* (): INTEGER;
        VAR d: MyDirectory; attr: TextRulers.Attributes;
            style: TextRulers.Style; ruler: TextRulers.Ruler;
            p: TextRulers.Prop;
            tabsBefore, tabsAfter: TextRulers.TabArray;
            packed, i: INTEGER;
    BEGIN
        NEW(d);
        NEW(attr);
        attr.first := 100;
        attr.left  := 200;
        attr.right := 300;

        (* Stage 1: Directory.NewStyle dispatches into our
           concrete MyDirectoryDesc.NewStyle via vtable. *)
        style := d.NewStyle(attr);
        IF style = NIL THEN RETURN -1 END;
        IF style.attr # attr THEN RETURN -2 END;

        (* Stage 2: Directory.New dispatches similarly. *)
        ruler := d.New(style);
        IF ruler = NIL THEN RETURN -3 END;
        IF ruler.style # style THEN RETURN -4 END;

        (* Stage 3: CopyTabs round-trips a TabArray containing
           an inline ARRAY 32 OF Tab.  Verifies per-element
           field access on a cross-module Named record (Tab
           is declared in TextRulers; that's the path #34
           closed). *)
        tabsBefore.len := 3;
        tabsBefore.tab[0].stop := 50;
        tabsBefore.tab[0].type := {TextRulers.centerTab};
        tabsBefore.tab[1].stop := 120;
        tabsBefore.tab[1].type := {TextRulers.rightTab};
        tabsBefore.tab[2].stop := 240;
        tabsBefore.tab[2].type := {};
        TextRulers.CopyTabs(tabsBefore, tabsAfter);
        IF tabsAfter.len # 3 THEN RETURN -5 END;
        IF tabsAfter.tab[0].stop # 50 THEN RETURN -6 END;
        IF ~(TextRulers.centerTab IN tabsAfter.tab[0].type) THEN RETURN -7 END;
        IF tabsAfter.tab[2].stop # 240 THEN RETURN -8 END;

        (* Stage 4: construct a Prop directly, set fields,
           verify they round-trip through the inline-record
           opts field (val + mask SETs). *)
        NEW(p);
        p.first := 100;
        p.left  := 200;
        p.right := 300;
        p.opts.val  := {TextRulers.leftAdjust, TextRulers.parJoin};
        p.opts.mask := {TextRulers.leftAdjust, TextRulers.rightAdjust,
                        TextRulers.parJoin};

        i := 0;
        IF TextRulers.leftAdjust IN p.opts.val  THEN INC(i) END;     (* +1 *)
        IF TextRulers.rightAdjust IN p.opts.val THEN INC(i) END;     (* 0 *)
        IF TextRulers.parJoin IN p.opts.val    THEN INC(i) END;      (* +1 *)
        IF TextRulers.parJoin IN p.opts.mask   THEN INC(i) END;      (* +1 *)
        (* expected i = 3 *)

        (* Pack:
             p.first (100) * 1000 + p.left (200) * 10 + i (3)
             + tabsAfter.tab[1].stop (120)
             = 100000 + 2000 + 3 + 120 = 102123 *)
        packed := p.first * 1000 + p.left * 10 + i + tabsAfter.tab[1].stop;
        RETURN packed       (* expected 102123 *)
    END Run;

END TextRulersExtBase.
