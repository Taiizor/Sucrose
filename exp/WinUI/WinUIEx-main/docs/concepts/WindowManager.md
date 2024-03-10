## Window Manager

The Window Manager is a helper for managing a window's size, position, backdrop. It's what is powering the WindowEx class, and can be
use to get the same features WindowEx provides but for any class. This can, for instance, be useful in conjunction with existing custom
Window classes, or with [.NET MAUI](Maui.md).

To create/get the manager, use the Get method
```cs
   var manager = WinUIEx.WindowManager.Get(window);
```

This allows you to set properties like Minimum size, backdrops, window position persistence etc.

```cs
manager.PersistenceId = "MainWindowPersistanceId";
manager.MinWidth = 640;
manager.MinHeight = 480;
manager.Backdrop = new WinUIEx.MicaSystemBackdrop();
```

