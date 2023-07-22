using System.Diagnostics;
using System.Windows;
using SSDEDT = Sucrose.Shared.Dependency.Enum.DisplayType;
using SESHD = Sucrose.Engine.Shared.Helper.Data;
using SWE = Skylark.Wing.Engine;
using SWHPI = Skylark.Wing.Helper.ProcessInterop;
using SWHWI = Skylark.Wing.Helper.WindowInterop;
using SWHWO = Skylark.Wing.Helper.WindowOperations;
using SWNM = Skylark.Wing.Native.Methods;

namespace Sucrose.Engine.Shared.Event
{
    internal static class Handler
    {
        public static void WindowLoaded(Window Window)
        {
            IntPtr Handle = SWHWI.Handle(Window);

            //ShowInTaskbar = false : causing issue with windows10-windows11 Taskview.
            SWHWO.RemoveWindowFromTaskbar(Handle);

            //this hides the window from taskbar and also fixes crash when win10-win11 taskview is launched. 
            Window.ShowInTaskbar = true;
            Window.ShowInTaskbar = false;
        }

        public static void ApplicationLoaded(Process Process)
        {
            IntPtr Handle = SWHPI.MainWindowHandle(Process);

            //ShowInTaskbar = false : causing issue with windows10-windows11 Taskview.
            SWHWO.RemoveWindowFromTaskbar(Handle);

            SWNM.ShowWindow(Handle, (int)SWNM.SHOWWINDOW.SW_HIDE);

            int currentStyle = SWNM.GetWindowLong(Handle, (int)SWNM.GWL.GWL_STYLE);
            SWNM.SetWindowLong(Handle, (int)SWNM.GWL.GWL_STYLE, currentStyle & ~((int)SWNM.WindowStyles.WS_CAPTION | (int)SWNM.WindowStyles.WS_THICKFRAME | (int)SWNM.WindowStyles.WS_MINIMIZE | (int)SWNM.WindowStyles.WS_MAXIMIZE | (int)SWNM.WindowStyles.WS_SYSMENU | (int)SWNM.WindowStyles.WS_DLGFRAME | (int)SWNM.WindowStyles.WS_BORDER | (int)SWNM.WindowStyles.WS_EX_CLIENTEDGE));

            SWHWO.BorderlessWinStyle(Handle);
        }

        public static void ContentRendered(Window Window)
        {
            switch (SESHD.GetDisplayType())
            {
                case SSDEDT.Expand:
                    SWE.WallpaperWindow(Window, SESHD.GetExpandScreenType(), SESHD.GetScreenType());
                    break;
                case SSDEDT.Duplicate:
                    SWE.WallpaperWindow(Window, SESHD.GetDuplicateScreenType(), SESHD.GetScreenType());
                    break;
                default:
                    SWE.WallpaperWindow(Window, SESHD.GetScreenIndex(), SESHD.GetScreenType());
                    break;
            }
        }

        public static void ApplicationRendered(Process Process)
        {
            switch (SESHD.GetDisplayType())
            {
                case SSDEDT.Expand:
                    SWE.WallpaperProcess(Process, SESHD.GetExpandScreenType(), SESHD.GetScreenType());
                    break;
                case SSDEDT.Duplicate:
                    //SWE.WallpaperProcess(Process, SESHD.GetDuplicateScreenType(), SESHD.GetScreenType());
                    break;
                default:
                    SWE.WallpaperProcess(Process, SESHD.GetScreenIndex(), SESHD.GetScreenType());
                    break;
            }

            SWNM.ShowWindow(SWHPI.MainWindowHandle(Process), (int)SWNM.SHOWWINDOW.SW_SHOW);
        }
    }
}