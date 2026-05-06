MODULE App;

IMPORT Factorial, Log, WinLoop, WinSpec, WinView;

PROCEDURE BuildWindow;
BEGIN
  WinSpec.Begin(WinView.title);
  WinSpec.OpenStack(-1);
  WinSpec.OpenRow(-1);
  WinSpec.AddButton("run_factorial", "Factorial 20", "run_factorial");
  WinSpec.AddButton("clear_log", "Clear", "clear_log");
  WinSpec.CloseContainer;
  WinSpec.AddTextarea("log", "Log", Log.text, 1);
  WinSpec.CloseContainer
END BuildWindow;

PROCEDURE OnClose*(name, payload: ARRAY OF SHORTCHAR);
BEGIN
END OnClose;

PROCEDURE Run*;
BEGIN
  Log.Open;
  Log.String("NewCP ready."); Log.Ln;
  WinView.SetTitle("NewCP");
  WinView.SetRenderer(BuildWindow);
  WinLoop.OnClose(OnClose);
  WinLoop.Register("run_factorial", Factorial.OnRun);
  WinLoop.Register("clear_log", Log.OnClear);
  WinView.Render;
  WinLoop.Run
END Run;

END App.
