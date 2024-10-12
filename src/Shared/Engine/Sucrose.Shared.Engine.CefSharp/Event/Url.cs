﻿using CefSharp;
using System.Windows;
using SEIT = Skylark.Enum.InputType;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSECSEI = Sucrose.Shared.Engine.CefSharp.Extension.Interaction;
using SSECSHH = Sucrose.Shared.Engine.CefSharp.Helper.Handle;
using SSECSHM = Sucrose.Shared.Engine.CefSharp.Helper.Management;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SMME = Sucrose.Manager.Manage.Engine;
using SMMCE = Sucrose.Memory.Manage.Constant.Engine;
using SMMMCE = Sucrose.Memory.Manage.Constant.Engine;

namespace Sucrose.Shared.Engine.CefSharp.Event
{
    internal static class Url
    {
        public static void CefEngineInitialized(object sender, EventArgs e)
        {
            SSECSHM.SetProcesses();
        }

        public static void CefEngineLoaded(object sender, RoutedEventArgs e)
        {
            SSECSMI.CefEngine.Address = SSECSMI.Url;
        }

        public static void CefEngineInitializedChanged(object sender, EventArgs e)
        {
            SSECSHH.GetInputHandle();

            SSECSHH.GetIntermediateHandle();

            if (SMME.InputType != SEIT.Close)
            {
                SSECSEI.Register();
            }

            if (SMME.DeveloperMode)
            {
                SSECSMI.CefEngine.ShowDevTools();
            }

            SSEMI.Initialized = SSECSMI.CefEngine.IsBrowserInitialized;
        }

        public static void CefEngineFrameLoadEnd(object sender, FrameLoadEndEventArgs e)
        {
            SSECSHM.SetProcesses();
        }
    }
}