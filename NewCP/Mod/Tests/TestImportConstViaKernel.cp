MODULE TestImportConstViaKernel;
IMPORT Kernel;
CONST
    derived = Kernel.timeResolution DIV 2;

PROCEDURE Run* (): INTEGER;
BEGIN
    RETURN derived
END Run;
END TestImportConstViaKernel.
