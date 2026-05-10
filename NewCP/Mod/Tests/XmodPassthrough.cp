MODULE XmodPassthrough;
(* Tiny module that wraps a definition-module call.  Used by
   tests::xmod_passthrough_compiles to verify a CP MODULE that
   delegates into a runtime-backed DEFINITION MODULE compiles
   cleanly.  This is the codegen path the typed Stores.Reader
   facade needs. *)

IMPORT StoresSys;

PROCEDURE OpenDoc* (IN path: ARRAY OF CHAR): INTEGER;
BEGIN RETURN StoresSys.OpenDocument(path) END OpenDoc;

END XmodPassthrough.
