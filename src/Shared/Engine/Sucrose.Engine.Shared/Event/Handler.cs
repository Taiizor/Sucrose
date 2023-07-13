using System.Diagnostics;
using System.Windows;
using SDEDT = Sucrose.Dependency.Enum.DisplayType;
using SEDST = Skylark.Enum.DuplicateScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SEST = Skylark.Enum.ScreenType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
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

            SWHWO.RemoveWindowFromTaskbar(Handle);

            int currentStyle = SWNM.GetWindowLong(Handle, (int)SWNM.GWL.GWL_STYLE);
            SWNM.SetWindowLong(Handle, (int)SWNM.GWL.GWL_STYLE, currentStyle & ~((int)SWNM.WindowStyles.WS_CAPTION | (int)SWNM.WindowStyles.WS_THICKFRAME | (int)SWNM.WindowStyles.WS_MINIMIZE | (int)SWNM.WindowStyles.WS_MAXIMIZE | (int)SWNM.WindowStyles.WS_SYSMENU | (int)SWNM.WindowStyles.WS_DLGFRAME | (int)SWNM.WindowStyles.WS_BORDER | (int)SWNM.WindowStyles.WS_EX_CLIENTEDGE));
        }

        public static void ContentRendered(Window Window)
        {
            switch (SMMI.EngineSettingManager.GetSetting(SMC.DisplayType, SDEDT.Screen))
            {
                case SDEDT.Expand:
                    SWE.WallpaperWindow(Window, SMMI.EngineSettingManager.GetSetting(SMC.ExpandScreenType, SEEST.Default), SMMI.EngineSettingManager.GetSetting(SMC.ScreenType, SEST.DisplayBound));
                    break;
                case SDEDT.Duplicate:
                    SWE.WallpaperWindow(Window, SMMI.EngineSettingManager.GetSetting(SMC.DuplicateScreenType, SEDST.Default), SMMI.EngineSettingManager.GetSetting(SMC.ScreenType, SEST.DisplayBound));
                    break;
                default:
                    SWE.WallpaperWindow(Window, SMMI.EngineSettingManager.GetSettingStable(SMC.ScreenIndex, 0), SMMI.EngineSettingManager.GetSetting(SMC.ScreenType, SEST.DisplayBound));
                    break;
            }
        }

        public static void ApplicationRendered(Process Process)
        {
            switch (SMMI.EngineSettingManager.GetSetting(SMC.DisplayType, SDEDT.Screen))
            {
                case SDEDT.Expand:
                    SWE.WallpaperProcess(Process, SMMI.EngineSettingManager.GetSetting(SMC.ExpandScreenType, SEEST.Default), SMMI.EngineSettingManager.GetSetting(SMC.ScreenType, SEST.DisplayBound));
                    break;
                case SDEDT.Duplicate:
                    //SWE.WallpaperProcess(Process, SMMI.EngineSettingManager.GetSetting(SMC.DuplicateScreenType, SEDST.Default), SMMI.EngineSettingManager.GetSetting(SMC.ScreenType, SEST.DisplayBound));
                    break;
                default:
                    SWE.WallpaperProcess(Process, SMMI.EngineSettingManager.GetSettingStable(SMC.ScreenIndex, 0), SMMI.EngineSettingManager.GetSetting(SMC.ScreenType, SEST.DisplayBound));
                    break;
            }
        }
    }
}