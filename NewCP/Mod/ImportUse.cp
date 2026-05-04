MODULE ImportUse;

IMPORT ImportBase;

PROCEDURE TwiceImported*(x: INTEGER): INTEGER;
BEGIN
    RETURN ImportBase.AddOne(ImportBase.AddOne(x))
END TwiceImported;

BEGIN
END ImportUse.