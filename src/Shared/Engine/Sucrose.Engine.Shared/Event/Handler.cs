using System.Diagnostics;
using System.Windows;
using SDEDT = Sucrose.Dependency.Enum.DisplayType;
using SEDST = Skylark.Enum.DuplicateScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SEST = Skylark.Enum.ScreenType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SWE = Skylark.Wing.Engine;
using SWHWI = Skylark.Wing.Helper.WindowInterop;
using SWHPI = Skylark.Wing.Helper.ProcessInterop;
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
            IntPtr Handle = SWHPI.Handle(Process);

            SWNM.ShowWindow(Handle, (int)SWNM.SHOWWINDOW.SW_HIDE);

            int exStyle = SWNM.GetWindowLong(Handle, (int)SWNM.GWL.GWL_EXSTYLE);
            SWNM.SetWindowLong(Handle, (int)SWNM.GWL.GWL_EXSTYLE, exStyle | (int)SWNM.WindowStyles.WS_EX_NOACTIVATE);
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