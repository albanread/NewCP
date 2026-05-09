MODULE TestAbstractExtend;

TYPE
  Base* = ABSTRACT RECORD END;
  Derived* = RECORD (Base) END;

END TestAbstractExtend.
