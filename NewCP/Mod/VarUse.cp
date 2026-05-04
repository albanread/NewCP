MODULE VarUse;

IMPORT VarBase;

PROCEDURE UseBump*(x: INTEGER): INTEGER;
BEGIN
    VarBase.Bump(x);
    RETURN x
END UseBump;

BEGIN
END VarUse.
