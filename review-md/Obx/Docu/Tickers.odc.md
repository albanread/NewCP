**Overview by Example: ObxTickers**

This example implements a simple stock ticker. The curve which is displayed shows a random walk. The black line is the origin, at which the walk started. If the curve reaches the upper or lower end of the view, then the origin is moved accordingly.



What this example shows is the use of actions. *Services.Actions*  are objects whose* Do*  procedures are executed in a delayed fashion, when the system is idle. An action which re-installs itself whenever it is invoked as in this example operates as a non-preemptive background task.

 "ObxTickers.Deposit; StdCmds.Open"

[<u>ObxTickers  sources</u>](../Mod/Tickers.odc.md)

