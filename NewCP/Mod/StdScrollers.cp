MODULE StdScrollers;
(*
   Scrollable view decorator.  `StdScrollers.View` wraps an inner
   `Views.View`, keeping a scroll origin (`org`, `orgX`) and
   forwarding Restore / model-change events to the wrapped view.
   `New` is the only public factory.
*)

    IMPORT Stores, Models, Views;

    CONST
        minVersion = 0;
        maxVersion = 0;

    TYPE
        StdScrollerObserverDesc = RECORD (Models.ModelObserverDesc)
            scroller: View
        END;
        StdScrollerObserver = POINTER TO StdScrollerObserverDesc;

        ViewDesc* = EXTENSIBLE RECORD (Views.ViewDesc)
            inner-: Views.View;
            org-:   INTEGER;
            orgX-:  INTEGER
        END;
        View* = POINTER TO ViewDesc;


    PROCEDURE (obs: StdScrollerObserver) Notify* (m: Models.Model; VAR msg: Models.Message);
    BEGIN
        obs.scroller.HandleModelMsg(msg)
    END Notify;


    PROCEDURE (v: View) Domain* (): Stores.Domain;
    BEGIN
        IF v.inner # NIL THEN RETURN v.inner.Domain()
        ELSE RETURN NIL
        END
    END Domain;

    PROCEDURE (v: View) Restore* (f: Views.Frame; l, t, r, b: INTEGER);
    BEGIN
        IF v.inner # NIL THEN
            v.inner.Restore(f, l, t, r, b)
        END
    END Restore;

    PROCEDURE (v: View) HandleModelMsg* (VAR msg: Models.Message);
    BEGIN
        Views.Update(v, Views.keepFrames)
    END HandleModelMsg;

    PROCEDURE (v: View) HandleViewMsg* (f: Views.Frame; VAR msg: Views.Message);
    BEGIN
        WITH msg: Views.UpdateCachesMsg DO
            v.Restore(f, f.l, f.t, f.r, f.b)
        ELSE
        END
    END HandleViewMsg;

    PROCEDURE (v: View) Externalize* (VAR wr: Stores.Writer), EXTENSIBLE;
    BEGIN
        v.Externalize^(wr);
        wr.WriteVersion(maxVersion);
        wr.WriteLong(v.org);
        wr.WriteLong(v.orgX);
        IF v.inner # NIL THEN
            wr.WriteBool(TRUE);
            wr.WriteStore(v.inner)
        ELSE
            wr.WriteBool(FALSE)
        END
    END Externalize;

    PROCEDURE (v: View) Internalize* (VAR rd: Stores.Reader), EXTENSIBLE;
        VAR ver, org, orgX: INTEGER; hasInner: BOOLEAN;
            handle: Stores.ReaderHandle; s: Stores.Store;
    BEGIN
        v.Internalize^(rd);
        rd.ReadVersion(minVersion, maxVersion, ver);
        IF rd.cancelled THEN RETURN END;
        rd.ReadLong(org);
        IF rd.eof THEN RETURN END;
        rd.ReadLong(orgX);
        IF rd.eof THEN RETURN END;
        v.org  := org;
        v.orgX := orgX;
        rd.ReadBool(hasInner);
        IF rd.eof THEN RETURN END;
        IF hasInner THEN
            rd.ReadStore(handle);
            IF rd.cancelled THEN RETURN END;
            IF handle # 0 THEN
                s := Stores.NewStore(handle);
                IF (s # NIL) & (s IS Views.View) THEN
                    v.inner := s(Views.View)
                END
            END
        END
    END Internalize;


    PROCEDURE New* (inner: Views.View): View;
        VAR v: View; obs: StdScrollerObserver; m: Models.Model;
    BEGIN
        NEW(v);
        v.inner := inner;
        v.org   := 0;
        v.orgX  := 0;
        IF inner # NIL THEN
            m := inner.ThisModel();
            IF m # NIL THEN
                NEW(obs);
                obs.scroller := v;
                Models.InstallObserver(m, obs)
            END
        END;
        RETURN v
    END New;

END StdScrollers.
