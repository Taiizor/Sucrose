## Window Message Monitor

The Windows Message Monitor allows you to receive raw Windows Messaging Events and further control and monitor the Window.

Example:
```cs 
var monitor = new WindowMessageMonitor(this);
monitor.WindowMessageReceived += OnWindowMessageReceived;
```

In the event handler you can then inspect these raw messages based on message id for type of message, and use the WParam and LParam according to the Windows Messaging documentation.
Example:

```cs
struct RECT
{
    public int left;
    public int top;
    public int right;
    public int bottom;
}

private void WindowMessageReceived(object sender, WindowMessageEventArgs e)
{
    if (e.Message.MessageId == 0x0214) //WM_SIZING event
    {
        // https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-sizing
        string side = "";
        switch (e.Message.WParam)
        {
            case 1: side = "Left"; break;
            case 2: side = "Right"; break;
            case 3: side = "Top"; break;
            case 4: side = "Top-Left"; break;
            case 5: side = "Top-Right"; break;
            case 6: side = "Bottom"; break;
            case 7: side = "Bottom-Left"; break;
            case 8: side = "Bottom-Right"; break;
        }
        var rect = Marshal.PtrToStructure<RECT>((IntPtr)e.Message.LParam);
        System.Diagnostics.Debug.WriteLine($"WM_SIZING: Side: {side} Rect: {rect.left},{rect.top},{rect.right},{rect.bottom}");
    }
}
```
