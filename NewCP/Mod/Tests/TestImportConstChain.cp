MODULE TestImportConstChain;
IMPORT Kernel;
CONST
    res = Kernel.timeResolution;
    half = res DIV 2;

PROCEDURE Run* (): INTEGER;
BEGIN
    RETURN half
END Run;
END TestImportConstChain.
