using Skylark.Wing.Helper;
using System.Windows;

namespace Sucrose.Player.Shared.Event
{
    internal static class Handler
    {
        public static void WindowLoaded(Window window)
        {
            IntPtr Handle = WindowInterop.Handle(window);

            //ShowInTaskbar = false : causing issue with windows10-windows11 Taskview.
            WindowOperations.RemoveWindowFromTaskbar(Handle);

            //this hides the window from taskbar and also fixes crash when win10-win11 taskview is launched. 
            window.ShowInTaskbar = true;
            window.ShowInTaskbar = false;
        }
    }
}