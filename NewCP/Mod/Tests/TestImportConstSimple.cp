MODULE TestImportConstSimple;
IMPORT Kernel;
CONST
    res = Kernel.timeResolution;

PROCEDURE Run* (): INTEGER;
BEGIN
    RETURN res
END Run;
END TestImportConstSimple.
