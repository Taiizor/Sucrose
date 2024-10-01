using System.Windows;
using SEDST = Skylark.Enum.DisplayScreenType;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandType;
using SSDSHS = Sucrose.Shared.Dependency.Struct.HandleStruct;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SWE = Skylark.Wing.Engine;
using SWHWI = Skylark.Wing.Helper.WindowInterop;
using SWHWO = Skylark.Wing.Helper.WindowOperations;
using SWNM = Skylark.Wing.Native.Methods;
using SWUS = Skylark.Wing.Utility.Screene;

namespace Sucrose.Shared.Engine.Event
{
    internal static class Handler
    {
        public static async void WindowLoaded(Window Window)
        {
            try
            {
                SSEMI.WindowHandle = SWHWI.Handle(Window);

                //ShowInTaskbar = false : causing issue with Windows10-Windows11 Taskview.
                SWHWO.RemoveWindowFromTaskbar(SSEMI.WindowHandle);

                //this hides the window from taskbar and also fixes crash when Win10-Win11 taskview is launched. 
                Window.ShowInTaskbar = true;
                Window.ShowInTaskbar = false;
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);

                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.RestartLive}{SMR.ValueSeparator}{SMR.Unknown}");
            }
        }

        public static async void EngineLoaded(IntPtr Engine)
        {
            try
            {
                //ShowInTaskbar = false : causing issue with Windows10-Windows11 Taskview.
                SWHWO.RemoveWindowFromTaskbar(Engine);

                SWNM.ShowWindow(Engine, (int)SWNM.SHOWWINDOW.SW_HIDE);

                int Style = SWNM.GetWindowLong(Engine, (int)SWNM.GWL.GWL_STYLE);
                SWNM.SetWindowLong(Engine, (int)SWNM.GWL.GWL_STYLE, Style & ~((int)SWNM.WindowStyles.WS_CAPTION | (int)SWNM.WindowStyles.WS_THICKFRAME | (int)SWNM.WindowStyles.WS_MINIMIZE | (int)SWNM.WindowStyles.WS_MAXIMIZE | (int)SWNM.WindowStyles.WS_SYSMENU | (int)SWNM.WindowStyles.WS_DLGFRAME | (int)SWNM.WindowStyles.WS_BORDER | (int)SWNM.WindowStyles.WS_EX_CLIENTEDGE));

                SWHWO.BorderlessWinStyle(Engine);
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);

                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.RestartLive}{SMR.ValueSeparator}{SMR.Unknown}");
            }
        }

        public static async void EngineRendered(IntPtr Handle)
        {
            try
            {
                switch (SSEHD.GetDisplayScreenType())
                {
                    case SEDST.SpanAcross:
                        SWE.WallpaperHandle(Handle, SSEHD.GetExpandScreenType(), SSEHD.GetScreenType());
                        break;
                    case SEDST.SameDuplicate:
                        SSEMI.Applications.ForEach(Application => SWE.WallpaperProcess(Application.Process, SSEMI.Applications.IndexOf(Application), SSEHD.GetScreenType()));
                        break;
                    default:
                        SWE.WallpaperHandle(Handle, SSEHD.GetScreenIndex(), SSEHD.GetScreenType());
                        break;
                }

                SWNM.ShowWindow(Handle, (int)SWNM.SHOWWINDOW.SW_SHOW);
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);

                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.RestartLive}{SMR.ValueSeparator}{SMR.Unknown}");
            }
        }

        public static async void ContentRendered(Window Window)
        {
            try
            {
                switch (SSEHD.GetDisplayScreenType())
                {
                    case SEDST.SpanAcross:
                        SWE.WallpaperWindow(Window, SSEHD.GetExpandScreenType(), SSEHD.GetScreenType());
                        break;
                    case SEDST.SameDuplicate:
                        SWE.WallpaperWindow(Window, SSEHD.GetDuplicateScreenType(), SSEHD.GetScreenType());
                        break;
                    default:
                        SWE.WallpaperWindow(Window, SSEHD.GetScreenIndex(), SSEHD.GetScreenType());
                        break;
                }
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);

                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.RestartLive}{SMR.ValueSeparator}{SMR.Unknown}");
            }
        }

        public static async void ApplicationLoaded(SSDSHS Application)
        {
            try
            {
                //ShowInTaskbar = false : causing issue with Windows10-Windows11 Taskview.
                SWHWO.RemoveWindowFromTaskbar(Application.Handle);
                SWHWO.RemoveWindowFromTaskbar(Application.MainWindowHandle);

                SWNM.ShowWindow(Application.Handle, (int)SWNM.SHOWWINDOW.SW_HIDE);
                SWNM.ShowWindow(Application.MainWindowHandle, (int)SWNM.SHOWWINDOW.SW_HIDE);

                int Style = SWNM.GetWindowLong(Application.MainWindowHandle, (int)SWNM.GWL.GWL_STYLE);
                SWNM.SetWindowLong(Application.MainWindowHandle, (int)SWNM.GWL.GWL_STYLE, Style & ~((int)SWNM.WindowStyles.WS_CAPTION | (int)SWNM.WindowStyles.WS_THICKFRAME | (int)SWNM.WindowStyles.WS_MINIMIZE | (int)SWNM.WindowStyles.WS_MAXIMIZE | (int)SWNM.WindowStyles.WS_SYSMENU | (int)SWNM.WindowStyles.WS_DLGFRAME | (int)SWNM.WindowStyles.WS_BORDER | (int)SWNM.WindowStyles.WS_EX_CLIENTEDGE));

                int MainWindowStyle = SWNM.GetWindowLong(Application.MainWindowHandle, (int)SWNM.GWL.GWL_STYLE);
                SWNM.SetWindowLong(Application.MainWindowHandle, (int)SWNM.GWL.GWL_STYLE, MainWindowStyle & ~((int)SWNM.WindowStyles.WS_CAPTION | (int)SWNM.WindowStyles.WS_THICKFRAME | (int)SWNM.WindowStyles.WS_MINIMIZE | (int)SWNM.WindowStyles.WS_MAXIMIZE | (int)SWNM.WindowStyles.WS_SYSMENU | (int)SWNM.WindowStyles.WS_DLGFRAME | (int)SWNM.WindowStyles.WS_BORDER | (int)SWNM.WindowStyles.WS_EX_CLIENTEDGE));

                SWHWO.BorderlessWinStyle(Application.Handle);
                SWHWO.BorderlessWinStyle(Application.MainWindowHandle);
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);

                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.RestartLive}{SMR.ValueSeparator}{SMR.Unknown}");
            }
        }

        public static async void DisplaySettingsChanged(Window Window)
        {
            try
            {
                SSEMI.Displaying?.Cancel();

                SSEMI.Displaying = new CancellationTokenSource();

                Window.Hide();

                try
                {
                    await Task.Delay(2000, SSEMI.Displaying.Token);

                    SWUS.Initialize();

                    await Task.Delay(500, SSEMI.Displaying.Token);

                    ContentRendered(Window);

                    Window.Show();
                }
                catch (TaskCanceledException) { }
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);

                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.RestartLive}{SMR.ValueSeparator}{SMR.Unknown}");
            }
        }

        public static async void ApplicationRendered(SSDSHS Application)
        {
            try
            {
                switch (SSEHD.GetDisplayScreenType())
                {
                    case SEDST.SpanAcross:
                        SWE.WallpaperProcess(Application.Process, SSEHD.GetExpandScreenType(), SSEHD.GetScreenType());
                        break;
                    case SEDST.SameDuplicate:
                        SSEMI.Applications.ForEach(Application => SWE.WallpaperProcess(Application.Process, SSEMI.Applications.IndexOf(Application), SSEHD.GetScreenType()));
                        break;
                    default:
                        SWE.WallpaperProcess(Application.Process, SSEHD.GetScreenIndex(), SSEHD.GetScreenType());
                        break;
                }

                SWNM.ShowWindow(Application.MainWindowHandle, (int)SWNM.SHOWWINDOW.SW_SHOW);
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);

                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.RestartLive}{SMR.ValueSeparator}{SMR.Unknown}");
            }
        }

        public static async void DisplaySettingsChanged(SSDSHS Application)
        {
            try
            {
                SSEMI.Displaying?.Cancel();

                SSEMI.Displaying = new CancellationTokenSource();

                SWNM.ShowWindow(Application.Handle, (int)SWNM.SHOWWINDOW.SW_HIDE);
                SWNM.ShowWindow(Application.MainWindowHandle, (int)SWNM.SHOWWINDOW.SW_HIDE);

                try
                {
                    await Task.Delay(2000, SSEMI.Displaying.Token);

                    SWUS.Initialize();

                    await Task.Delay(500, SSEMI.Displaying.Token);

                    ApplicationRendered(Application);

                    SWNM.ShowWindow(Application.Handle, (int)SWNM.SHOWWINDOW.SW_SHOW);
                    SWNM.ShowWindow(Application.MainWindowHandle, (int)SWNM.SHOWWINDOW.SW_SHOW);
                }
                catch (TaskCanceledException) { }
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);

                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.RestartLive}{SMR.ValueSeparator}{SMR.Unknown}");
            }
        }
    }
}