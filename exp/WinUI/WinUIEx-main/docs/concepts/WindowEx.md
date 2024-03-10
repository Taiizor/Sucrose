## Extended Window type

Make your window class inherit from `WinUIEx.WindowEx` instead of `Window` and you get new features like:
 - Set Title in XAML.
 - Width and Height get/set properties, and correctly adjusting for system DPI.
 - Minimum Width and Height get/set properties.
 - Set TaskBarIcon.
 - Several Convenience properties for controlling if the window can be maximized, minimized, resized, should show in task bar and switchers, change presenter,  make Window always on top, etc.
 - Easy shortcuts for showing Message Dialogs.
 - Automatically handles DPI changes and resizes correctly.

## Usage

To use the WindowEx class, change the baseclass of your window (usually MainWindow).
In `MainWindow.xaml.cs` change 
```cs
  public sealed partial class MainWindow : Window
```
to
```cs
  public sealed partial class MainWindow : WinUIEx.WindowEx
```
And in `MainWindow.xaml` change
```xml
<Window   ...>
```
to
```xml
<winex:WindowEx xmlns:winex="using:WinUIEx" Width="1024" Height="768"  ...>
```

## Object Model
![image](https://user-images.githubusercontent.com/1378165/170792197-0cfd9c54-6682-4b82-98fd-9f4e069db599.png)

