**Overview by Example: ObxActions**

This example demonstrates how background tasks can be implemented using *actions*. An action is an object which performs some action later when the system is idle, i.e., between user interactions. An action can be scheduled to execute as soon as possible, or after some time has passed. Upon execution, an action may re-schedule itself for a later point in time. In this way, an action can operate as a background task, getting computation time whenever the system is idle. This strategy is called *cooperative multitasking*. For this to work, an action must not do massive computations, because this would reduce the responsiveness of the system. Longer calculations need to be broken down into less time consuming pieces. This is demonstrated by an algorithm which calculates prime numbers up to a given maximum, as a background task. An action is used to perform the stepwise calculation. Every newly found prime number is written into a text. This text remains invisible as long as the calculation goes on. The action checks whether it has reached a maximum set by the user. If this is not yet the case, it re-schedules itself for further execution. Otherwise, it opens a window with the list of prime numbers, i.e., a text view on the created text.

 "StdCmds.OpenAuxDialog('Obx/Rsrc/Actions', 'Prime Calculation')"

[<u>ObxActions  sources</u>](../Mod/Actions.odc.md)

