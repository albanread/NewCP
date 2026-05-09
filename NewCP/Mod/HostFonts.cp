MODULE HostFonts;

IMPORT Fonts;

TYPE
  DirectoryDesc* = RECORD (Fonts.DirectoryDesc) END;
  Directory*     = POINTER TO DirectoryDesc;

END HostFonts.
