## Window Extension methods

To get the extension methods, first add `using WinUIEx;` to the top of your code.

- Minimize/Maximize/Restore and Hide window.
```cs
    myWindow.Minimize();
    myWindow.Maximize();
    myWindow.Restore();
    myWindow.Hide();`
```

- Move and resize window
```cs
   myWindow.CenterOnScreen();
   myWindow.SetWindowSize(1024, 768);
   myWindow.MoveAndResize(100, 100, 1024, 768);
```

- Make Window always-on-top
```cs
    myWindow.SetIsAlwaysOnTop(true);
```

- Bring window to the top
```cs
    myWindow.BringToFront();
```
![image](https://user-images.githubusercontent.com/1378165/145075953-2ec1e01e-525a-49da-8454-e78724a382fb.png)