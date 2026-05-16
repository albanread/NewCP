MODULE HostWindowsSys;
(*
   Sys layer between HostWindows and iGui.  Only module that imports iGui for
   window-title management.  Character conversion: CHAR (16-bit CP string) →
   SHORTCHAR (8-bit iGui WINAPI string).
*)

    IMPORT iGui;

    PROCEDURE SetTitle* (childId: INTEGER; IN title: ARRAY OF CHAR);
        VAR shortTitle: ARRAY 256 OF SHORTCHAR;
            i: INTEGER;
    BEGIN
        i := 0;
        WHILE (title[i] # 0X) & (i < 255) DO
            shortTitle[i] := SHORT(title[i]);
            INC(i)
        END;
        shortTitle[i] := 0X;
        iGui.SetTitle(childId, shortTitle)
    END SetTitle;

END HostWindowsSys.
